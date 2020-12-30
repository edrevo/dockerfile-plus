use std::collections::HashMap;
use std::sync::Arc;

use buildkit_proto::pb::{self, op::Op, OpMetadata, SourceOp};

use crate::ops::{OperationBuilder, SingleBorrowedOutput, SingleOwnedOutput};
use crate::serialization::{Context, Node, Operation, OperationId, Result};
use crate::utils::{OperationOutput, OutputIdx};

#[derive(Default, Debug)]
pub struct HttpSource {
    id: OperationId,
    url: String,
    file_name: Option<String>,
    description: HashMap<String, String>,
    ignore_cache: bool,
}

impl HttpSource {
    pub(crate) fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: OperationId::default(),
            url: url.into(),
            file_name: None,
            description: Default::default(),
            ignore_cache: false,
        }
    }
}

impl HttpSource {
    pub fn with_file_name<S>(mut self, name: S) -> Self
    where
        S: Into<String>,
    {
        self.file_name = Some(name.into());
        self
    }
}

impl<'a> SingleBorrowedOutput<'a> for HttpSource {
    fn output(&'a self) -> OperationOutput<'a> {
        OperationOutput::borrowed(self, OutputIdx(0))
    }
}

impl<'a> SingleOwnedOutput<'static> for Arc<HttpSource> {
    fn output(&self) -> OperationOutput<'static> {
        OperationOutput::owned(self.clone(), OutputIdx(0))
    }
}

impl OperationBuilder<'static> for HttpSource {
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

impl Operation for HttpSource {
    fn id(&self) -> &OperationId {
        &self.id
    }

    fn serialize(&self, _: &mut Context) -> Result<Node> {
        let mut attrs = HashMap::default();

        if let Some(ref file_name) = self.file_name {
            attrs.insert("http.filename".into(), file_name.into());
        }

        let head = pb::Op {
            op: Some(Op::Source(SourceOp {
                identifier: self.url.clone(),
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
        HttpSource::new("http://any.url/with/path"),
        |digest| { "sha256:22ec64461f39dd3b54680fc240b459248b1ced597f113b5d692abe9695860d12" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "http://any.url/with/path".into(),
                attrs: Default::default(),
            })
        },
    );

    crate::check_op!(
        HttpSource::new("http://any.url/with/path").custom_name("git custom name"),
        |digest| { "sha256:22ec64461f39dd3b54680fc240b459248b1ced597f113b5d692abe9695860d12" },
        |description| { vec![("llb.customname", "git custom name")] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "http://any.url/with/path".into(),
                attrs: Default::default(),
            })
        },
    );

    crate::check_op!(
        HttpSource::new("http://any.url/with/path").with_file_name("file.name"),
        |digest| { "sha256:e1fe6584287dfa2b065ed29fcf4f77bcf86fb54781832d2f45074fa1671df692" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "http://any.url/with/path".into(),
                attrs: vec![("http.filename".to_string(), "file.name".to_string())]
                    .into_iter()
                    .collect(),
            })
        },
    );
}
