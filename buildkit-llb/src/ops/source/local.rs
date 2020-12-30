use std::collections::HashMap;
use std::sync::Arc;

use buildkit_proto::pb::{self, op::Op, OpMetadata, SourceOp};

use crate::ops::{OperationBuilder, SingleBorrowedOutput, SingleOwnedOutput};
use crate::serialization::{Context, Node, Operation, OperationId, Result};
use crate::utils::{OperationOutput, OutputIdx};

#[derive(Default, Debug)]
pub struct LocalSource {
    id: OperationId,
    name: String,
    description: HashMap<String, String>,
    ignore_cache: bool,

    exclude: Vec<String>,
    include: Vec<String>,
}

impl LocalSource {
    pub(crate) fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: OperationId::default(),
            name: name.into(),
            ignore_cache: false,

            ..Default::default()
        }
    }

    pub fn add_include_pattern<S>(mut self, include: S) -> Self
    where
        S: Into<String>,
    {
        // TODO: add `source.local.includepatterns` capability
        self.include.push(include.into());
        self
    }

    pub fn add_exclude_pattern<S>(mut self, exclude: S) -> Self
    where
        S: Into<String>,
    {
        // TODO: add `source.local.excludepatterns` capability
        self.exclude.push(exclude.into());
        self
    }
}

impl<'a> SingleBorrowedOutput<'a> for LocalSource {
    fn output(&'a self) -> OperationOutput<'a> {
        OperationOutput::borrowed(self, OutputIdx(0))
    }
}

impl<'a> SingleOwnedOutput<'static> for Arc<LocalSource> {
    fn output(&self) -> OperationOutput<'static> {
        OperationOutput::owned(self.clone(), OutputIdx(0))
    }
}

impl OperationBuilder<'static> for LocalSource {
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

impl Operation for LocalSource {
    fn id(&self) -> &OperationId {
        &self.id
    }

    fn serialize(&self, _: &mut Context) -> Result<Node> {
        let mut attrs = HashMap::default();

        if !self.exclude.is_empty() {
            attrs.insert(
                "local.excludepatterns".into(),
                serde_json::to_string(&self.exclude).unwrap(),
            );
        }

        if !self.include.is_empty() {
            attrs.insert(
                "local.includepattern".into(),
                serde_json::to_string(&self.include).unwrap(),
            );
        }

        let head = pb::Op {
            op: Some(Op::Source(SourceOp {
                identifier: format!("local://{}", self.name),
                attrs,
            })),

            ..Default::default()
        };

        let metadata = OpMetadata {
            description: self.description.clone(),
            ignore_cache: self.ignore_cache,

            ..Default::default()
        };

        Ok(Node::new(head, metadata))
    }
}

#[test]
fn serialization() {
    crate::check_op!(
        LocalSource::new("context"),
        |digest| { "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "local://context".into(),
                attrs: Default::default(),
            })
        },
    );

    crate::check_op!(
        LocalSource::new("context").custom_name("context custom name"),
        |digest| { "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702" },
        |description| { vec![("llb.customname", "context custom name")] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "local://context".into(),
                attrs: Default::default(),
            })
        },
    );

    crate::check_op!(
        {
            LocalSource::new("context")
                .custom_name("context custom name")
                .add_exclude_pattern("**/target")
                .add_exclude_pattern("Dockerfile")
        },
        |digest| { "sha256:f6962b8bb1659c63a2c2c3e2a7ccf0326c87530dd70c514343f127e4c20460c4" },
        |description| { vec![("llb.customname", "context custom name")] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "local://context".into(),
                attrs: crate::utils::test::to_map(vec![(
                    "local.excludepatterns",
                    r#"["**/target","Dockerfile"]"#,
                )]),
            })
        },
    );

    crate::check_op!(
        {
            LocalSource::new("context")
                .custom_name("context custom name")
                .add_include_pattern("Cargo.toml")
                .add_include_pattern("inner/Cargo.toml")
        },
        |digest| { "sha256:a7e628333262b810572f83193bbf8554e688abfb51d44ac30bdad7fa425f3839" },
        |description| { vec![("llb.customname", "context custom name")] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "local://context".into(),
                attrs: crate::utils::test::to_map(vec![(
                    "local.includepattern",
                    r#"["Cargo.toml","inner/Cargo.toml"]"#,
                )]),
            })
        },
    );
}
