use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use buildkit_proto::pb::{self, op::Op, OpMetadata, SourceOp};
use lazy_static::*;
use regex::Regex;

use crate::ops::{OperationBuilder, SingleBorrowedOutput, SingleOwnedOutput};
use crate::serialization::{Context, Node, Operation, OperationId, Result};
use crate::utils::{OperationOutput, OutputIdx};

#[derive(Debug)]
pub struct ImageSource {
    id: OperationId,

    domain: Option<String>,
    name: String,
    tag: Option<String>,
    digest: Option<String>,

    description: HashMap<String, String>,
    ignore_cache: bool,
    resolve_mode: Option<ResolveMode>,
}

#[derive(Debug, Clone, Copy)]
pub enum ResolveMode {
    Default,
    ForcePull,
    PreferLocal,
}

impl fmt::Display for ResolveMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResolveMode::Default => write!(f, "default"),
            ResolveMode::ForcePull => write!(f, "pull"),
            ResolveMode::PreferLocal => write!(f, "local"),
        }
    }
}

impl Default for ResolveMode {
    fn default() -> Self {
        ResolveMode::Default
    }
}

lazy_static! {
    static ref TAG_EXPR: Regex = Regex::new(r":[\w][\w.-]+$").unwrap();
}

impl ImageSource {
    // The implementation is based on:
    // https://github.com/containerd/containerd/blob/614c0858f2a8db9ee0c788a9164870069f3e53ed/reference/docker/reference.go
    pub(crate) fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        let mut name = name.into();

        let (digest, digest_separator) = match name.find('@') {
            Some(pos) => (Some(name[pos + 1..].into()), pos),
            None => (None, name.len()),
        };

        name.truncate(digest_separator);

        let (tag, tag_separator) = match TAG_EXPR.find(&name) {
            Some(found) => (Some(name[found.start() + 1..].into()), found.start()),
            None => (None, name.len()),
        };

        name.truncate(tag_separator);

        let (domain, mut name) = match name.find('/') {
            // The input has canonical-like format.
            Some(separator_pos) if &name[..separator_pos] == "docker.io" => {
                (None, name[separator_pos + 1..].into())
            }

            // Special case when domain is "localhost".
            Some(separator_pos) if &name[..separator_pos] == "localhost" => {
                (Some("localhost".into()), name[separator_pos + 1..].into())
            }

            // General case for a common domain.
            Some(separator_pos) if name[..separator_pos].find('.').is_some() => (
                Some(name[..separator_pos].into()),
                name[separator_pos + 1..].into(),
            ),

            // General case for a domain with port number.
            Some(separator_pos) if name[..separator_pos].find(':').is_some() => (
                Some(name[..separator_pos].into()),
                name[separator_pos + 1..].into(),
            ),

            // Fallback if the first component is not a domain name.
            Some(_) => (None, name),

            // Fallback if only single url component present.
            None => (None, name),
        };

        if domain.is_none() && name.find('/').is_none() {
            name = format!("library/{}", name);
        }

        Self {
            id: OperationId::default(),

            domain,
            name,
            tag,
            digest,

            description: Default::default(),
            ignore_cache: false,
            resolve_mode: None,
        }
    }

    pub fn with_resolve_mode(mut self, mode: ResolveMode) -> Self {
        self.resolve_mode = Some(mode);
        self
    }

    pub fn resolve_mode(&self) -> Option<ResolveMode> {
        self.resolve_mode
    }

    pub fn with_digest<S>(mut self, digest: S) -> Self
    where
        S: Into<String>,
    {
        self.digest = Some(digest.into());
        self
    }

    pub fn with_tag<S>(mut self, tag: S) -> Self
    where
        S: Into<String>,
    {
        self.tag = Some(tag.into());
        self
    }

    pub fn canonical_name(&self) -> String {
        let domain = match self.domain {
            Some(ref domain) => domain,
            None => "docker.io",
        };

        let tag = match self.tag {
            Some(ref tag) => tag,
            None => "latest",
        };

        match self.digest {
            Some(ref digest) => format!("{}/{}:{}@{}", domain, self.name, tag, digest),
            None => format!("{}/{}:{}", domain, self.name, tag),
        }
    }
}

impl<'a> SingleBorrowedOutput<'a> for ImageSource {
    fn output(&'a self) -> OperationOutput<'a> {
        OperationOutput::borrowed(self, OutputIdx(0))
    }
}

impl<'a> SingleOwnedOutput<'static> for Arc<ImageSource> {
    fn output(&self) -> OperationOutput<'static> {
        OperationOutput::owned(self.clone(), OutputIdx(0))
    }
}

impl OperationBuilder<'static> for ImageSource {
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

impl Operation for ImageSource {
    fn id(&self) -> &OperationId {
        &self.id
    }

    fn serialize(&self, _: &mut Context) -> Result<Node> {
        let mut attrs = HashMap::default();

        if let Some(ref mode) = self.resolve_mode {
            attrs.insert("image.resolvemode".into(), mode.to_string());
        }

        let head = pb::Op {
            op: Some(Op::Source(SourceOp {
                identifier: format!("docker-image://{}", self.canonical_name()),
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
        ImageSource::new("rustlang/rust:nightly"),
        |digest| { "sha256:dee2a3d7dd482dd8098ba543ff1dcb01efd29fcd16fdb0979ef556f38564543a" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "docker-image://docker.io/rustlang/rust:nightly".into(),
                attrs: Default::default(),
            })
        },
    );

    crate::check_op!(
        ImageSource::new("library/alpine:latest"),
        |digest| { "sha256:0e6b31ceed3e6dc542018f35a53a0e857e6a188453d32a2a5bbe7aa2971c1220" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "docker-image://docker.io/library/alpine:latest".into(),
                attrs: Default::default(),
            })
        },
    );

    crate::check_op!(
        ImageSource::new("rustlang/rust:nightly").custom_name("image custom name"),
        |digest| { "sha256:dee2a3d7dd482dd8098ba543ff1dcb01efd29fcd16fdb0979ef556f38564543a" },
        |description| { vec![("llb.customname", "image custom name")] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "docker-image://docker.io/rustlang/rust:nightly".into(),
                attrs: Default::default(),
            })
        },
    );

    crate::check_op!(
        ImageSource::new("rustlang/rust:nightly").with_digest("sha256:123456"),
        |digest| { "sha256:a9837e26998d165e7b6433f8d40b36d259905295860fcbbc62bbce75a6c991c6" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "docker-image://docker.io/rustlang/rust:nightly@sha256:123456".into(),
                attrs: Default::default(),
            })
        },
    );
}

#[test]
fn resolve_mode() {
    crate::check_op!(
        ImageSource::new("rustlang/rust:nightly").with_resolve_mode(ResolveMode::Default),
        |digest| { "sha256:792e246751e84b9a5e40c28900d70771a07e8cc920c1039cdddfc6bf69256dfe" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "docker-image://docker.io/rustlang/rust:nightly".into(),
                attrs: crate::utils::test::to_map(vec![("image.resolvemode", "default")]),
            })
        },
    );

    crate::check_op!(
        ImageSource::new("rustlang/rust:nightly").with_resolve_mode(ResolveMode::ForcePull),
        |digest| { "sha256:0bd920010eab701bdce44c61d220e6943d56d3fb9a9fa4e773fc060c0d746122" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "docker-image://docker.io/rustlang/rust:nightly".into(),
                attrs: crate::utils::test::to_map(vec![("image.resolvemode", "pull")]),
            })
        },
    );

    crate::check_op!(
        ImageSource::new("rustlang/rust:nightly").with_resolve_mode(ResolveMode::PreferLocal),
        |digest| { "sha256:bd6797c8644d2663b29c36a8b3b63931e539be44ede5e56aca2da4f35f241f18" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Source(SourceOp {
                identifier: "docker-image://docker.io/rustlang/rust:nightly".into(),
                attrs: crate::utils::test::to_map(vec![("image.resolvemode", "local")]),
            })
        },
    );
}

#[test]
fn image_name() {
    crate::check_op!(ImageSource::new("rustlang/rust"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://docker.io/rustlang/rust:latest".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("rust:nightly"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://docker.io/library/rust:nightly".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("rust"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://docker.io/library/rust:latest".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("library/rust"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://docker.io/library/rust:latest".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("rust:obj@sha256:abcdef"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://docker.io/library/rust:obj@sha256:abcdef".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("rust@sha256:abcdef"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://docker.io/library/rust:latest@sha256:abcdef".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("rust:obj@abcdef"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://docker.io/library/rust:obj@abcdef".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(
        ImageSource::new("b.gcr.io/test.example.com/my-app:test.example.com"),
        |op| {
            Op::Source(SourceOp {
                identifier: "docker-image://b.gcr.io/test.example.com/my-app:test.example.com"
                    .into(),
                attrs: Default::default(),
            })
        }
    );

    crate::check_op!(
        ImageSource::new("sub-dom1.foo.com/bar/baz/quux:some-long-tag"),
        |op| {
            Op::Source(SourceOp {
                identifier: "docker-image://sub-dom1.foo.com/bar/baz/quux:some-long-tag".into(),
                attrs: Default::default(),
            })
        }
    );

    crate::check_op!(
        ImageSource::new("sub-dom1.foo.com/quux:some-long-tag"),
        |op| {
            Op::Source(SourceOp {
                identifier: "docker-image://sub-dom1.foo.com/quux:some-long-tag".into(),
                attrs: Default::default(),
            })
        }
    );

    crate::check_op!(ImageSource::new("localhost/rust:obj"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://localhost/rust:obj".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("127.0.0.1/rust:obj"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://127.0.0.1/rust:obj".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("localhost:5000/rust:obj"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://localhost:5000/rust:obj".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("127.0.0.1:5000/rust:obj"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://127.0.0.1:5000/rust:obj".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("localhost:5000/rust"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://localhost:5000/rust:latest".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("127.0.0.1:5000/rust"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://127.0.0.1:5000/rust:latest".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("docker.io/rust"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://docker.io/library/rust:latest".into(),
            attrs: Default::default(),
        })
    });

    crate::check_op!(ImageSource::new("docker.io/library/rust"), |op| {
        Op::Source(SourceOp {
            identifier: "docker-image://docker.io/library/rust:latest".into(),
            attrs: Default::default(),
        })
    });
}
