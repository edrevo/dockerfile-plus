use std::collections::HashMap;
use std::sync::Arc;

use buildkit_proto::pb::{self, op::Op, OpMetadata, SourceOp};

use crate::ops::{OperationBuilder, SingleBorrowedOutput, SingleOwnedOutput};
use crate::serialization::{Context, Node, Operation, OperationId, Result};
use crate::utils::{OperationOutput, OutputIdx};

#[derive(Default, Debug)]
pub struct GitSource {
    id: OperationId,
    remote: String,
    reference: Option<String>,
    description: HashMap<String, String>,
    ignore_cache: bool,
}

impl GitSource {
    pub(crate) fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        let mut raw_url = url.into();
        let remote = if raw_url.starts_with("http://") {
            raw_url.split_off(7)
        } else if raw_url.starts_with("https://") {
            raw_url.split_off(8)
        } else if raw_url.starts_with("git://") {
            raw_url.split_off(6)
        } else if raw_url.starts_with("git@") {
            raw_url.split_off(4)
        } else {
            raw_url
        };

        Self {
            id: OperationId::default(),
            remote,
            reference: None,
            description: Default::default(),
            ignore_cache: false,
        }
    }
}

impl GitSource {
    pub fn with_reference<S>(mut self, reference: S) -> Self
    where
        S: Into<String>,
    {
        self.reference = Some(reference.into());
        self
    }
}

impl<'a> SingleBorrowedOutput<'a> for GitSource {
    fn output(&'a self) -> OperationOutput<'a> {
        OperationOutput::borrowed(self, OutputIdx(0))
    }
}

impl<'a> SingleOwnedOutput<'static> for Arc<GitSource> {
    fn output(&self) -> OperationOutput<'static> {
        OperationOutput::owned(self.clone(), OutputIdx(0))
    }
}

impl OperationBuilder<'static> for GitSource {
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

impl Operation for GitSource {
    fn id(&self) -> &OperationId {
        &self.id
    }

    fn serialize(&self, _: &mut Context) -> Result<Node> {
        let identifier = if let Some(ref reference) = self.reference {
            format!("git://{}#{}", self.remote, reference)
        } else {
            format!("git://{}", self.remote)
        };

        let head = pb::Op {
            op: Some(Op::Source(SourceOp {
                identifier,
                attrs: Default::default(),
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
        GitSource::new("any.url"),
        |digest| { "sha256:ecde982e19ace932e5474e57b0ca71ba690ed7d28abff2a033e8f969e22bf2d8" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "git://any.url".into(),
                attrs: Default::default(),
            })
        },
    );

    crate::check_op!(
        GitSource::new("any.url").custom_name("git custom name"),
        |digest| { "sha256:ecde982e19ace932e5474e57b0ca71ba690ed7d28abff2a033e8f969e22bf2d8" },
        |description| { vec![("llb.customname", "git custom name")] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "git://any.url".into(),
                attrs: Default::default(),
            })
        },
    );
}

#[test]
fn prefixes() {
    crate::check_op!(
        GitSource::new("http://any.url"),
        |digest| { "sha256:ecde982e19ace932e5474e57b0ca71ba690ed7d28abff2a033e8f969e22bf2d8" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "git://any.url".into(),
                attrs: Default::default(),
            })
        },
    );

    crate::check_op!(
        GitSource::new("https://any.url"),
        |digest| { "sha256:ecde982e19ace932e5474e57b0ca71ba690ed7d28abff2a033e8f969e22bf2d8" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "git://any.url".into(),
                attrs: Default::default(),
            })
        },
    );

    crate::check_op!(
        GitSource::new("git://any.url"),
        |digest| { "sha256:ecde982e19ace932e5474e57b0ca71ba690ed7d28abff2a033e8f969e22bf2d8" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "git://any.url".into(),
                attrs: Default::default(),
            })
        },
    );

    crate::check_op!(
        GitSource::new("git@any.url"),
        |digest| { "sha256:ecde982e19ace932e5474e57b0ca71ba690ed7d28abff2a033e8f969e22bf2d8" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "git://any.url".into(),
                attrs: Default::default(),
            })
        },
    );
}

#[test]
fn with_reference() {
    crate::check_op!(
        GitSource::new("any.url").with_reference("abcdef"),
        |digest| { "sha256:f59aa7f8db62e0b5c2a1da396752ba8a2bb0b5d28ddcfdd1d4f822d26ebfe3cf" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "git://any.url#abcdef".into(),
                attrs: Default::default(),
            })
        },
    );
}
