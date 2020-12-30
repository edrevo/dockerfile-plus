use std::collections::HashMap;
use std::sync::Arc;

use buildkit_proto::pb::{self, op::Op};

use super::FileOperation;

use crate::ops::*;
use crate::serialization::{Context, Node, Operation, OperationId, Result};
use crate::utils::{OperationOutput, OutputIdx};

#[derive(Debug)]
pub struct SequenceOperation<'a> {
    id: OperationId,
    inner: Vec<Box<dyn FileOperation + 'a>>,

    description: HashMap<String, String>,
    caps: HashMap<String, bool>,
    ignore_cache: bool,
}

impl<'a> SequenceOperation<'a> {
    pub(crate) fn new() -> Self {
        let mut caps = HashMap::<String, bool>::new();
        caps.insert("file.base".into(), true);

        Self {
            id: OperationId::default(),
            inner: vec![],

            caps,
            description: Default::default(),
            ignore_cache: false,
        }
    }

    pub fn append<T>(mut self, op: T) -> Self
    where
        T: FileOperation + 'a,
    {
        // TODO: verify no duplicated outputs

        self.inner.push(Box::new(op));
        self
    }

    pub fn last_output_index(&self) -> Option<u32> {
        // TODO: make sure the `inner` elements have monotonic indexes

        self.inner
            .iter()
            .filter(|fs| fs.output() >= 0)
            .last()
            .map(|fs| fs.output() as u32)
    }
}

impl<'a, 'b: 'a> MultiBorrowedOutput<'b> for SequenceOperation<'b> {
    fn output(&'b self, index: u32) -> OperationOutput<'b> {
        // TODO: check if the requested index available.
        OperationOutput::borrowed(self, OutputIdx(index))
    }
}

impl<'a> MultiOwnedOutput<'a> for Arc<SequenceOperation<'a>> {
    fn output(&self, index: u32) -> OperationOutput<'a> {
        // TODO: check if the requested index available.
        OperationOutput::owned(self.clone(), OutputIdx(index))
    }
}

impl<'a, 'b: 'a> MultiBorrowedLastOutput<'b> for SequenceOperation<'b> {
    fn last_output(&'b self) -> Option<OperationOutput<'b>> {
        self.last_output_index().map(|index| self.output(index))
    }
}

impl<'a> MultiOwnedLastOutput<'a> for Arc<SequenceOperation<'a>> {
    fn last_output(&self) -> Option<OperationOutput<'a>> {
        self.last_output_index().map(|index| self.output(index))
    }
}

impl<'a> OperationBuilder<'a> for SequenceOperation<'a> {
    fn custom_name<S>(mut self, name: S) -> Self
    where
        S: Into<String>,
    {
        self.description
            .insert("llb.customname".into(), name.into());

        self
    }

    fn ignore_cache(mut self, ignore: bool) -> Self {
        self.ignore_cache = ignore;
        self
    }
}

impl<'a> Operation for SequenceOperation<'a> {
    fn id(&self) -> &OperationId {
        &self.id
    }

    fn serialize(&self, cx: &mut Context) -> Result<Node> {
        let mut inputs = vec![];
        let mut input_offsets = vec![];

        for item in &self.inner {
            let mut inner_inputs = item.serialize_inputs(cx)?;

            input_offsets.push(inputs.len());
            inputs.append(&mut inner_inputs);
        }

        let mut actions = vec![];

        for (item, offset) in self.inner.iter().zip(input_offsets.into_iter()) {
            actions.push(item.serialize_action(inputs.len(), offset)?);
        }

        let head = pb::Op {
            inputs,
            op: Some(Op::File(pb::FileOp { actions })),

            ..Default::default()
        };

        let metadata = pb::OpMetadata {
            description: self.description.clone(),
            caps: self.caps.clone(),
            ignore_cache: self.ignore_cache,

            ..Default::default()
        };

        Ok(Node::new(head, metadata))
    }
}
