use std::fmt::Debug;
use std::path::Path;

use buildkit_proto::pb;

use crate::serialization::{Context, Result};
use crate::utils::OutputIdx;

mod copy;
mod mkdir;
mod mkfile;
mod path;
mod sequence;

pub use self::copy::CopyOperation;
pub use self::mkdir::MakeDirOperation;
pub use self::mkfile::MakeFileOperation;
pub use self::path::{LayerPath, UnsetPath};
pub use self::sequence::SequenceOperation;

/// Umbrella operation that handles file system related routines.
/// Dockerfile's `COPY` directive is a partial case of this.
pub struct FileSystem;

impl FileSystem {
    pub fn sequence() -> SequenceOperation<'static> {
        SequenceOperation::new()
    }

    pub fn copy() -> copy::CopyOperation<UnsetPath, UnsetPath> {
        CopyOperation::new()
    }

    pub fn mkdir<P>(output: OutputIdx, layer: LayerPath<P>) -> MakeDirOperation
    where
        P: AsRef<Path>,
    {
        MakeDirOperation::new(output, layer)
    }

    pub fn mkfile<P>(output: OutputIdx, layer: LayerPath<P>) -> MakeFileOperation
    where
        P: AsRef<Path>,
    {
        MakeFileOperation::new(output, layer)
    }
}

pub trait FileOperation: Debug + Send + Sync {
    fn output(&self) -> i32;

    fn serialize_inputs(&self, cx: &mut Context) -> Result<Vec<pb::Input>>;
    fn serialize_action(&self, inputs_count: usize, inputs_offset: usize)
        -> Result<pb::FileAction>;
}

#[test]
fn copy_serialization() {
    use crate::prelude::*;
    use buildkit_proto::pb::{file_action::Action, op::Op, FileAction, FileActionCopy, FileOp};

    let context = Source::local("context");
    let builder_image = Source::image("rustlang/rust:nightly");

    let operation = FileSystem::sequence()
        .append(
            FileSystem::copy()
                .from(LayerPath::Other(context.output(), "Cargo.toml"))
                .to(OutputIdx(0), LayerPath::Scratch("Cargo.toml")),
        )
        .append(
            FileSystem::copy()
                .from(LayerPath::Other(builder_image.output(), "/bin/sh"))
                .to(OutputIdx(1), LayerPath::Own(OwnOutputIdx(0), "/bin/sh")),
        )
        .append(
            FileSystem::copy()
                .from(LayerPath::Own(OwnOutputIdx(1), "Cargo.toml"))
                .to(OutputIdx(2), LayerPath::Scratch("Cargo.toml")),
        );

    crate::check_op!(
        operation,
        |digest| { "sha256:c4f7fb723fa87f03788aaf660dc9110ad8748fc9971e13713f103b632c05ae96" },
        |description| { vec![] },
        |caps| { vec!["file.base"] },
        |cached_tail| {
            vec![
                "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702",
                "sha256:dee2a3d7dd482dd8098ba543ff1dcb01efd29fcd16fdb0979ef556f38564543a",
            ]
        },
        |inputs| {
            vec![
                (
                    "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702",
                    0,
                ),
                (
                    "sha256:dee2a3d7dd482dd8098ba543ff1dcb01efd29fcd16fdb0979ef556f38564543a",
                    0,
                ),
            ]
        },
        |op| {
            Op::File(FileOp {
                actions: vec![
                    FileAction {
                        input: -1,
                        secondary_input: 0,
                        output: 0,
                        action: Some(Action::Copy(FileActionCopy {
                            src: "Cargo.toml".into(),
                            dest: "Cargo.toml".into(),
                            owner: None,
                            mode: -1,
                            follow_symlink: false,
                            dir_copy_contents: false,
                            attempt_unpack_docker_compatibility: false,
                            create_dest_path: false,
                            allow_wildcard: false,
                            allow_empty_wildcard: false,
                            timestamp: -1,
                        })),
                    },
                    FileAction {
                        input: 2,
                        secondary_input: 1,
                        output: 1,
                        action: Some(Action::Copy(FileActionCopy {
                            src: "/bin/sh".into(),
                            dest: "/bin/sh".into(),
                            owner: None,
                            mode: -1,
                            follow_symlink: false,
                            dir_copy_contents: false,
                            attempt_unpack_docker_compatibility: false,
                            create_dest_path: false,
                            allow_wildcard: false,
                            allow_empty_wildcard: false,
                            timestamp: -1,
                        })),
                    },
                    FileAction {
                        input: -1,
                        secondary_input: 3,
                        output: 2,
                        action: Some(Action::Copy(FileActionCopy {
                            src: "Cargo.toml".into(),
                            dest: "Cargo.toml".into(),
                            owner: None,
                            mode: -1,
                            follow_symlink: false,
                            dir_copy_contents: false,
                            attempt_unpack_docker_compatibility: false,
                            create_dest_path: false,
                            allow_wildcard: false,
                            allow_empty_wildcard: false,
                            timestamp: -1,
                        })),
                    },
                ],
            })
        },
    );
}

#[test]
fn copy_with_params_serialization() {
    use crate::prelude::*;
    use buildkit_proto::pb::{file_action::Action, op::Op, FileAction, FileActionCopy, FileOp};

    let context = Source::local("context");

    let operation = FileSystem::sequence()
        .append(
            FileSystem::copy()
                .from(LayerPath::Other(context.output(), "Cargo.toml"))
                .to(OutputIdx(0), LayerPath::Scratch("Cargo.toml"))
                .follow_symlinks(true),
        )
        .append(
            FileSystem::copy()
                .from(LayerPath::Other(context.output(), "Cargo.toml"))
                .to(OutputIdx(1), LayerPath::Scratch("Cargo.toml"))
                .recursive(true),
        )
        .append(
            FileSystem::copy()
                .from(LayerPath::Other(context.output(), "Cargo.toml"))
                .to(OutputIdx(2), LayerPath::Scratch("Cargo.toml"))
                .create_path(true),
        )
        .append(
            FileSystem::copy()
                .from(LayerPath::Other(context.output(), "Cargo.toml"))
                .to(OutputIdx(3), LayerPath::Scratch("Cargo.toml"))
                .wildcard(true),
        );

    crate::check_op!(
        operation,
        |digest| { "sha256:8be9c1c8335d53c894d0f5848ef354c69a96a469a72b00aadae704b23d465022" },
        |description| { vec![] },
        |caps| { vec!["file.base"] },
        |cached_tail| {
            vec!["sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702"]
        },
        |inputs| {
            // TODO: improve the correct, but inefficent serialization
            vec![
                (
                    "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702",
                    0,
                ),
                (
                    "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702",
                    0,
                ),
                (
                    "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702",
                    0,
                ),
                (
                    "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702",
                    0,
                ),
            ]
        },
        |op| {
            Op::File(FileOp {
                actions: vec![
                    FileAction {
                        input: -1,
                        secondary_input: 0,
                        output: 0,
                        action: Some(Action::Copy(FileActionCopy {
                            src: "Cargo.toml".into(),
                            dest: "Cargo.toml".into(),
                            owner: None,
                            mode: -1,
                            follow_symlink: true,
                            dir_copy_contents: false,
                            attempt_unpack_docker_compatibility: false,
                            create_dest_path: false,
                            allow_wildcard: false,
                            allow_empty_wildcard: false,
                            timestamp: -1,
                        })),
                    },
                    FileAction {
                        input: -1,
                        secondary_input: 1,
                        output: 1,
                        action: Some(Action::Copy(FileActionCopy {
                            src: "Cargo.toml".into(),
                            dest: "Cargo.toml".into(),
                            owner: None,
                            mode: -1,
                            follow_symlink: false,
                            dir_copy_contents: true,
                            attempt_unpack_docker_compatibility: false,
                            create_dest_path: false,
                            allow_wildcard: false,
                            allow_empty_wildcard: false,
                            timestamp: -1,
                        })),
                    },
                    FileAction {
                        input: -1,
                        secondary_input: 2,
                        output: 2,
                        action: Some(Action::Copy(FileActionCopy {
                            src: "Cargo.toml".into(),
                            dest: "Cargo.toml".into(),
                            owner: None,
                            mode: -1,
                            follow_symlink: false,
                            dir_copy_contents: false,
                            attempt_unpack_docker_compatibility: false,
                            create_dest_path: true,
                            allow_wildcard: false,
                            allow_empty_wildcard: false,
                            timestamp: -1,
                        })),
                    },
                    FileAction {
                        input: -1,
                        secondary_input: 3,
                        output: 3,
                        action: Some(Action::Copy(FileActionCopy {
                            src: "Cargo.toml".into(),
                            dest: "Cargo.toml".into(),
                            owner: None,
                            mode: -1,
                            follow_symlink: false,
                            dir_copy_contents: false,
                            attempt_unpack_docker_compatibility: false,
                            create_dest_path: false,
                            allow_wildcard: true,
                            allow_empty_wildcard: false,
                            timestamp: -1,
                        })),
                    },
                ],
            })
        },
    );
}

#[test]
fn mkdir_serialization() {
    use crate::prelude::*;
    use buildkit_proto::pb::{file_action::Action, op::Op, FileAction, FileActionMkDir, FileOp};

    let context = Source::local("context");

    let operation = FileSystem::sequence()
        .append(
            FileSystem::mkdir(
                OutputIdx(0),
                LayerPath::Other(context.output(), "/new-crate"),
            )
            .make_parents(true),
        )
        .append(FileSystem::mkdir(
            OutputIdx(1),
            LayerPath::Scratch("/new-crate"),
        ))
        .append(FileSystem::mkdir(
            OutputIdx(2),
            LayerPath::Own(OwnOutputIdx(1), "/another-crate/deep/directory"),
        ));

    crate::check_op!(
        operation,
        |digest| { "sha256:bfcd58256cba441c6d9e89c439bc6640b437d47213472cf8491646af4f0aa5b2" },
        |description| { vec![] },
        |caps| { vec!["file.base"] },
        |cached_tail| {
            vec!["sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702"]
        },
        |inputs| {
            vec![(
                "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702",
                0,
            )]
        },
        |op| {
            Op::File(FileOp {
                actions: vec![
                    FileAction {
                        input: 0,
                        secondary_input: -1,
                        output: 0,
                        action: Some(Action::Mkdir(FileActionMkDir {
                            path: "/new-crate".into(),
                            owner: None,
                            mode: -1,
                            timestamp: -1,
                            make_parents: true,
                        })),
                    },
                    FileAction {
                        input: -1,
                        secondary_input: -1,
                        output: 1,
                        action: Some(Action::Mkdir(FileActionMkDir {
                            path: "/new-crate".into(),
                            owner: None,
                            mode: -1,
                            timestamp: -1,
                            make_parents: false,
                        })),
                    },
                    FileAction {
                        input: 2,
                        secondary_input: -1,
                        output: 2,
                        action: Some(Action::Mkdir(FileActionMkDir {
                            path: "/another-crate/deep/directory".into(),
                            owner: None,
                            mode: -1,
                            timestamp: -1,
                            make_parents: false,
                        })),
                    },
                ],
            })
        },
    );
}

#[test]
fn mkfile_serialization() {
    use crate::prelude::*;
    use buildkit_proto::pb::{file_action::Action, op::Op, FileAction, FileActionMkFile, FileOp};

    let context = Source::local("context");

    let operation = FileSystem::sequence()
        .append(
            FileSystem::mkfile(
                OutputIdx(0),
                LayerPath::Other(context.output(), "/build-plan.json"),
            )
            .data(b"any bytes".to_vec()),
        )
        .append(FileSystem::mkfile(
            OutputIdx(1),
            LayerPath::Scratch("/build-graph.json"),
        ))
        .append(FileSystem::mkfile(
            OutputIdx(2),
            LayerPath::Own(OwnOutputIdx(1), "/llb.pb"),
        ));

    crate::check_op!(
        operation,
        |digest| { "sha256:9c0d9f741dfc9b4ea8d909ebf388bc354da0ee401eddf5633e8e4ece7e87d22d" },
        |description| { vec![] },
        |caps| { vec!["file.base"] },
        |cached_tail| {
            vec!["sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702"]
        },
        |inputs| {
            vec![(
                "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702",
                0,
            )]
        },
        |op| {
            Op::File(FileOp {
                actions: vec![
                    FileAction {
                        input: 0,
                        secondary_input: -1,
                        output: 0,
                        action: Some(Action::Mkfile(FileActionMkFile {
                            path: "/build-plan.json".into(),
                            owner: None,
                            mode: -1,
                            timestamp: -1,
                            data: b"any bytes".to_vec(),
                        })),
                    },
                    FileAction {
                        input: -1,
                        secondary_input: -1,
                        output: 1,
                        action: Some(Action::Mkfile(FileActionMkFile {
                            path: "/build-graph.json".into(),
                            owner: None,
                            mode: -1,
                            timestamp: -1,
                            data: vec![],
                        })),
                    },
                    FileAction {
                        input: 2,
                        secondary_input: -1,
                        output: 2,
                        action: Some(Action::Mkfile(FileActionMkFile {
                            path: "/llb.pb".into(),
                            owner: None,
                            mode: -1,
                            timestamp: -1,
                            data: vec![],
                        })),
                    },
                ],
            })
        },
    );
}
