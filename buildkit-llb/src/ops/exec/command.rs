use std::collections::HashMap;
use std::iter::{empty, once};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use buildkit_proto::pb::{
    self, op::Op, ExecOp, Input, MountType, NetMode, OpMetadata, SecurityMode,
};
use either::Either;

use super::context::Context;
use super::mount::Mount;

use crate::ops::{MultiBorrowedOutput, MultiOwnedOutput, OperationBuilder};
use crate::serialization::{Context as SerializationCtx, Node, Operation, OperationId, Result};
use crate::utils::{OperationOutput, OutputIdx};

/// Command execution operation. This is what a Dockerfile's `RUN` directive is translated to.
#[derive(Debug, Clone)]
pub struct Command<'a> {
    id: OperationId,

    context: Context,
    root_mount: Option<Mount<'a, PathBuf>>,
    other_mounts: Vec<Mount<'a, PathBuf>>,

    description: HashMap<String, String>,
    caps: HashMap<String, bool>,
    ignore_cache: bool,
}

impl<'a> Command<'a> {
    pub fn run<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: OperationId::default(),

            context: Context::new(name),
            root_mount: None,
            other_mounts: vec![],

            description: Default::default(),
            caps: Default::default(),
            ignore_cache: false,
        }
    }

    pub fn args<A, S>(mut self, args: A) -> Self
    where
        A: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.context.args = args.into_iter().map(|item| item.as_ref().into()).collect();
        self
    }

    pub fn env<S, Q>(mut self, name: S, value: Q) -> Self
    where
        S: AsRef<str>,
        Q: AsRef<str>,
    {
        let env = format!("{}={}", name.as_ref(), value.as_ref());

        self.context.env.push(env);
        self
    }

    pub fn env_iter<I, S, Q>(mut self, iter: I) -> Self
    where
        I: IntoIterator<Item = (S, Q)>,
        S: AsRef<str>,
        Q: AsRef<str>,
    {
        for (name, value) in iter.into_iter() {
            let env = format!("{}={}", name.as_ref(), value.as_ref());
            self.context.env.push(env);
        }

        self
    }

    pub fn cwd<P>(mut self, path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        self.context.cwd = path.into();
        self
    }

    pub fn user<S>(mut self, user: S) -> Self
    where
        S: Into<String>,
    {
        self.context.user = user.into();
        self
    }

    pub fn mount<P>(mut self, mount: Mount<'a, P>) -> Self
    where
        P: AsRef<Path>,
    {
        match mount {
            Mount::Layer(..) | Mount::ReadOnlyLayer(..) | Mount::Scratch(..) => {
                self.caps.insert("exec.mount.bind".into(), true);
            }

            Mount::ReadOnlySelector(..) => {
                self.caps.insert("exec.mount.bind".into(), true);
                self.caps.insert("exec.mount.selector".into(), true);
            }

            Mount::SharedCache(..) => {
                self.caps.insert("exec.mount.cache".into(), true);
                self.caps.insert("exec.mount.cache.sharing".into(), true);
            }

            Mount::OptionalSshAgent(..) => {
                self.caps.insert("exec.mount.ssh".into(), true);
            }
        }

        if mount.is_root() {
            self.root_mount = Some(mount.into_owned());
        } else {
            self.other_mounts.push(mount.into_owned());
        }

        self
    }
}

impl<'a, 'b: 'a> MultiBorrowedOutput<'b> for Command<'b> {
    fn output(&'b self, index: u32) -> OperationOutput<'b> {
        // TODO: check if the requested index available.
        OperationOutput::borrowed(self, OutputIdx(index))
    }
}

impl<'a> MultiOwnedOutput<'a> for Arc<Command<'a>> {
    fn output(&self, index: u32) -> OperationOutput<'a> {
        // TODO: check if the requested index available.
        OperationOutput::owned(self.clone(), OutputIdx(index))
    }
}

impl<'a> OperationBuilder<'a> for Command<'a> {
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

impl<'a> Operation for Command<'a> {
    fn id(&self) -> &OperationId {
        &self.id
    }

    fn serialize(&self, cx: &mut SerializationCtx) -> Result<Node> {
        let (inputs, mounts): (Vec<_>, Vec<_>) = {
            let mut last_input_index = 0;

            self.root_mount
                .as_ref()
                .into_iter()
                .chain(self.other_mounts.iter())
                .map(|mount| {
                    let inner_mount = match mount {
                        Mount::ReadOnlyLayer(_, destination) => pb::Mount {
                            input: last_input_index,
                            dest: destination.to_string_lossy().into(),
                            output: -1,
                            readonly: true,
                            mount_type: MountType::Bind as i32,

                            ..Default::default()
                        },

                        Mount::ReadOnlySelector(_, destination, source) => pb::Mount {
                            input: last_input_index,
                            dest: destination.to_string_lossy().into(),
                            output: -1,
                            readonly: true,
                            selector: source.to_string_lossy().into(),
                            mount_type: MountType::Bind as i32,

                            ..Default::default()
                        },

                        Mount::Layer(output, _, path) => pb::Mount {
                            input: last_input_index,
                            dest: path.to_string_lossy().into(),
                            output: output.into(),
                            mount_type: MountType::Bind as i32,

                            ..Default::default()
                        },

                        Mount::Scratch(output, path) => {
                            let mount = pb::Mount {
                                input: -1,
                                dest: path.to_string_lossy().into(),
                                output: output.into(),
                                mount_type: MountType::Bind as i32,

                                ..Default::default()
                            };

                            return (Either::Right(empty()), mount);
                        }

                        Mount::SharedCache(path) => {
                            use buildkit_proto::pb::{CacheOpt, CacheSharingOpt};

                            let mount = pb::Mount {
                                input: -1,
                                dest: path.to_string_lossy().into(),
                                output: -1,
                                mount_type: MountType::Cache as i32,

                                cache_opt: Some(CacheOpt {
                                    id: path.display().to_string(),
                                    sharing: CacheSharingOpt::Shared as i32,
                                }),

                                ..Default::default()
                            };

                            return (Either::Right(empty()), mount);
                        }

                        Mount::OptionalSshAgent(path) => {
                            use buildkit_proto::pb::SshOpt;

                            let mount = pb::Mount {
                                input: -1,
                                dest: path.to_string_lossy().into(),
                                output: -1,
                                mount_type: MountType::Ssh as i32,

                                ssh_opt: Some(SshOpt {
                                    mode: 0o600,
                                    optional: true,
                                    ..Default::default()
                                }),

                                ..Default::default()
                            };

                            return (Either::Right(empty()), mount);
                        }
                    };

                    let input = match mount {
                        Mount::ReadOnlyLayer(input, ..) => input,
                        Mount::ReadOnlySelector(input, ..) => input,
                        Mount::Layer(_, input, ..) => input,

                        Mount::SharedCache(..) => {
                            unreachable!();
                        }

                        Mount::Scratch(..) => {
                            unreachable!();
                        }

                        Mount::OptionalSshAgent(..) => {
                            unreachable!();
                        }
                    };

                    let serialized = cx.register(input.operation()).unwrap();
                    let input = Input {
                        digest: serialized.digest.clone(),
                        index: input.output().into(),
                    };

                    last_input_index += 1;

                    (Either::Left(once(input)), inner_mount)
                })
                .unzip()
        };

        let head = pb::Op {
            op: Some(Op::Exec(ExecOp {
                mounts,
                network: NetMode::Unset.into(),
                security: SecurityMode::Sandbox.into(),
                meta: Some(self.context.clone().into()),
            })),

            inputs: inputs.into_iter().flatten().collect(),

            ..Default::default()
        };

        let metadata = OpMetadata {
            description: self.description.clone(),
            caps: self.caps.clone(),
            ignore_cache: self.ignore_cache,

            ..Default::default()
        };

        Ok(Node::new(head, metadata))
    }
}
