mod command;
mod context;
mod mount;

pub use command::Command;
pub use mount::Mount;

#[test]
fn serialization() {
    use crate::prelude::*;
    use buildkit_proto::pb::{op::Op, ExecOp, Meta, NetMode, SecurityMode};

    crate::check_op!(
        {
            Command::run("/bin/sh")
                .args(&["-c", "echo 'test string' > /out/file0"])
                .env("HOME", "/root")
                .custom_name("exec custom name")
        },
        |digest| { "sha256:dc9a5a3cd84bb1c7b633f1750fdfccd9d0a69d060f8e3babb297bc190e2d7484" },
        |description| { vec![("llb.customname", "exec custom name")] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Exec(ExecOp {
                mounts: vec![],
                network: NetMode::Unset.into(),
                security: SecurityMode::Sandbox.into(),
                meta: Some(Meta {
                    args: crate::utils::test::to_vec(vec![
                        "/bin/sh",
                        "-c",
                        "echo 'test string' > /out/file0",
                    ]),

                    env: crate::utils::test::to_vec(vec!["HOME=/root"]),
                    cwd: "/".into(),
                    user: "root".into(),

                    extra_hosts: vec![],
                    proxy_env: None,
                }),
            })
        },
    );
}

#[test]
fn serialization_with_env_iter() {
    use crate::prelude::*;
    use buildkit_proto::pb::{op::Op, ExecOp, Meta, NetMode, SecurityMode};

    crate::check_op!(
        {
            Command::run("cargo").args(&["build"]).env_iter(vec![
                ("HOME", "/root"),
                ("PATH", "/bin"),
                ("CARGO_HOME", "/root/.cargo"),
            ])
        },
        |digest| { "sha256:7675be0b02acb379d57bafee5dc749fca7e795fb1e0a92748ccc59a7bc3b491e" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Exec(ExecOp {
                mounts: vec![],
                network: NetMode::Unset.into(),
                security: SecurityMode::Sandbox.into(),
                meta: Some(Meta {
                    args: crate::utils::test::to_vec(vec!["cargo", "build"]),
                    env: crate::utils::test::to_vec(vec![
                        "HOME=/root",
                        "PATH=/bin",
                        "CARGO_HOME=/root/.cargo",
                    ]),

                    cwd: "/".into(),
                    user: "root".into(),

                    extra_hosts: vec![],
                    proxy_env: None,
                }),
            })
        },
    );
}

#[test]
fn serialization_with_cwd() {
    use crate::prelude::*;
    use buildkit_proto::pb::{op::Op, ExecOp, Meta, NetMode, SecurityMode};

    crate::check_op!(
        Command::run("cargo").args(&["build"]).cwd("/rust-src"),
        |digest| { "sha256:b8120a0e1d1f7fcaa3d6c95db292d064524dc92c6cae8b97672d4e1eafcd03fa" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Exec(ExecOp {
                mounts: vec![],
                network: NetMode::Unset.into(),
                security: SecurityMode::Sandbox.into(),
                meta: Some(Meta {
                    args: crate::utils::test::to_vec(vec!["cargo", "build"]),
                    env: vec![],
                    cwd: "/rust-src".into(),
                    user: "root".into(),

                    extra_hosts: vec![],
                    proxy_env: None,
                }),
            })
        },
    );
}

#[test]
fn serialization_with_user() {
    use crate::prelude::*;
    use buildkit_proto::pb::{op::Op, ExecOp, Meta, NetMode, SecurityMode};

    crate::check_op!(
        Command::run("cargo").args(&["build"]).user("builder"),
        |digest| { "sha256:7631ea645e2126e9dbc5d9ae789e34301d9d5c80ce89bfa72bc9b82aa43b57c0" },
        |description| { vec![] },
        |caps| { vec![] },
        |cached_tail| { vec![] },
        |inputs| { vec![] },
        |op| {
            Op::Exec(ExecOp {
                mounts: vec![],
                network: NetMode::Unset.into(),
                security: SecurityMode::Sandbox.into(),
                meta: Some(Meta {
                    args: crate::utils::test::to_vec(vec!["cargo", "build"]),
                    env: vec![],
                    cwd: "/".into(),
                    user: "builder".into(),

                    extra_hosts: vec![],
                    proxy_env: None,
                }),
            })
        },
    );
}

#[test]
fn serialization_with_mounts() {
    use crate::prelude::*;
    use buildkit_proto::pb::{
        op::Op, CacheOpt, CacheSharingOpt, ExecOp, Meta, MountType, NetMode, SecurityMode,
    };

    let context = Source::local("context");
    let builder_image = Source::image("rustlang/rust:nightly");
    let final_image = Source::image("library/alpine:latest");

    let command = Command::run("cargo")
        .args(&["build"])
        .mount(Mount::ReadOnlyLayer(builder_image.output(), "/"))
        .mount(Mount::Scratch(OutputIdx(1), "/tmp"))
        .mount(Mount::ReadOnlySelector(
            context.output(),
            "/buildkit-frontend",
            "/frontend-sources",
        ))
        .mount(Mount::Layer(OutputIdx(0), final_image.output(), "/output"))
        .mount(Mount::SharedCache("/root/.cargo"));

    crate::check_op!(
        command,
        |digest| { "sha256:54a66b514361b13b17f8b5aaaa2392a4c07b55ac53303e4f50584f3dfef6add0" },
        |description| { vec![] },
        |caps| {
            vec![
                "exec.mount.bind",
                "exec.mount.cache",
                "exec.mount.cache.sharing",
                "exec.mount.selector",
            ]
        },
        |cached_tail| {
            vec![
                "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702",
                "sha256:dee2a3d7dd482dd8098ba543ff1dcb01efd29fcd16fdb0979ef556f38564543a",
                "sha256:0e6b31ceed3e6dc542018f35a53a0e857e6a188453d32a2a5bbe7aa2971c1220",
            ]
        },
        |inputs| {
            vec![
                (
                    "sha256:dee2a3d7dd482dd8098ba543ff1dcb01efd29fcd16fdb0979ef556f38564543a",
                    0,
                ),
                (
                    "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702",
                    0,
                ),
                (
                    "sha256:0e6b31ceed3e6dc542018f35a53a0e857e6a188453d32a2a5bbe7aa2971c1220",
                    0,
                ),
            ]
        },
        |op| {
            Op::Exec(ExecOp {
                mounts: vec![
                    pb::Mount {
                        input: 0,
                        selector: "".into(),
                        dest: "/".into(),
                        output: -1,
                        readonly: true,
                        mount_type: MountType::Bind.into(),
                        cache_opt: None,
                        secret_opt: None,
                        ssh_opt: None,
                    },
                    pb::Mount {
                        input: -1,
                        selector: "".into(),
                        dest: "/tmp".into(),
                        output: 1,
                        readonly: false,
                        mount_type: MountType::Bind.into(),
                        cache_opt: None,
                        secret_opt: None,
                        ssh_opt: None,
                    },
                    pb::Mount {
                        input: 1,
                        selector: "/frontend-sources".into(),
                        dest: "/buildkit-frontend".into(),
                        output: -1,
                        readonly: true,
                        mount_type: MountType::Bind.into(),
                        cache_opt: None,
                        secret_opt: None,
                        ssh_opt: None,
                    },
                    pb::Mount {
                        input: 2,
                        selector: "".into(),
                        dest: "/output".into(),
                        output: 0,
                        readonly: false,
                        mount_type: MountType::Bind.into(),
                        cache_opt: None,
                        secret_opt: None,
                        ssh_opt: None,
                    },
                    pb::Mount {
                        input: -1,
                        selector: "".into(),
                        dest: "/root/.cargo".into(),
                        output: -1,
                        readonly: false,
                        mount_type: MountType::Cache.into(),
                        cache_opt: Some(CacheOpt {
                            id: "/root/.cargo".into(),
                            sharing: CacheSharingOpt::Shared.into(),
                        }),
                        secret_opt: None,
                        ssh_opt: None,
                    },
                ],
                network: NetMode::Unset.into(),
                security: SecurityMode::Sandbox.into(),
                meta: Some(Meta {
                    args: crate::utils::test::to_vec(vec!["cargo", "build"]),
                    env: vec![],
                    cwd: "/".into(),
                    user: "root".into(),

                    extra_hosts: vec![],
                    proxy_env: None,
                }),
            })
        },
    );
}

#[test]
fn serialization_with_several_root_mounts() {
    use crate::prelude::*;
    use buildkit_proto::pb::{op::Op, ExecOp, Meta, MountType, NetMode, SecurityMode};

    let builder_image = Source::image("rustlang/rust:nightly");
    let final_image = Source::image("library/alpine:latest");

    let command = Command::run("cargo")
        .args(&["build"])
        .mount(Mount::Scratch(OutputIdx(0), "/tmp"))
        .mount(Mount::ReadOnlyLayer(builder_image.output(), "/"))
        .mount(Mount::Scratch(OutputIdx(1), "/var"))
        .mount(Mount::ReadOnlyLayer(final_image.output(), "/"));

    crate::check_op!(
        command,
        |digest| { "sha256:baa1bf591d2c47058b7361a0284fa8a3f1bd0fac8a93c87affa77ddc0a5026fd" },
        |description| { vec![] },
        |caps| { vec!["exec.mount.bind"] },
        |cached_tail| {
            vec!["sha256:0e6b31ceed3e6dc542018f35a53a0e857e6a188453d32a2a5bbe7aa2971c1220"]
        },
        |inputs| {
            vec![(
                "sha256:0e6b31ceed3e6dc542018f35a53a0e857e6a188453d32a2a5bbe7aa2971c1220",
                0,
            )]
        },
        |op| {
            Op::Exec(ExecOp {
                mounts: vec![
                    pb::Mount {
                        input: 0,
                        selector: "".into(),
                        dest: "/".into(),
                        output: -1,
                        readonly: true,
                        mount_type: MountType::Bind.into(),
                        cache_opt: None,
                        secret_opt: None,
                        ssh_opt: None,
                    },
                    pb::Mount {
                        input: -1,
                        selector: "".into(),
                        dest: "/tmp".into(),
                        output: 0,
                        readonly: false,
                        mount_type: MountType::Bind.into(),
                        cache_opt: None,
                        secret_opt: None,
                        ssh_opt: None,
                    },
                    pb::Mount {
                        input: -1,
                        selector: "".into(),
                        dest: "/var".into(),
                        output: 1,
                        readonly: false,
                        mount_type: MountType::Bind.into(),
                        cache_opt: None,
                        secret_opt: None,
                        ssh_opt: None,
                    },
                ],
                network: NetMode::Unset.into(),
                security: SecurityMode::Sandbox.into(),
                meta: Some(Meta {
                    args: crate::utils::test::to_vec(vec!["cargo", "build"]),
                    env: vec![],
                    cwd: "/".into(),
                    user: "root".into(),

                    extra_hosts: vec![],
                    proxy_env: None,
                }),
            })
        },
    );
}

#[test]
fn serialization_with_ssh_mounts() {
    use crate::prelude::*;
    use buildkit_proto::pb::{op::Op, ExecOp, Meta, MountType, NetMode, SecurityMode, SshOpt};

    let builder_image = Source::image("rustlang/rust:nightly");
    let command = Command::run("cargo")
        .args(&["build"])
        .mount(Mount::ReadOnlyLayer(builder_image.output(), "/"))
        .mount(Mount::OptionalSshAgent("/run/buildkit/ssh_agent.0"));

    crate::check_op!(
        command,
        |digest| { "sha256:1ac1438c67a153878f21fe8067383fd7544901261374eb53ba8bf26e9a5821a5" },
        |description| { vec![] },
        |caps| { vec!["exec.mount.bind", "exec.mount.ssh"] },
        |cached_tail| {
            vec!["sha256:dee2a3d7dd482dd8098ba543ff1dcb01efd29fcd16fdb0979ef556f38564543a"]
        },
        |inputs| {
            vec![(
                "sha256:dee2a3d7dd482dd8098ba543ff1dcb01efd29fcd16fdb0979ef556f38564543a",
                0,
            )]
        },
        |op| {
            Op::Exec(ExecOp {
                mounts: vec![
                    pb::Mount {
                        input: 0,
                        selector: "".into(),
                        dest: "/".into(),
                        output: -1,
                        readonly: true,
                        mount_type: MountType::Bind.into(),
                        cache_opt: None,
                        secret_opt: None,
                        ssh_opt: None,
                    },
                    pb::Mount {
                        input: -1,
                        selector: "".into(),
                        dest: "/run/buildkit/ssh_agent.0".into(),
                        output: -1,
                        readonly: false,
                        mount_type: MountType::Ssh.into(),
                        cache_opt: None,
                        secret_opt: None,
                        ssh_opt: Some(SshOpt {
                            mode: 0o600,
                            optional: true,
                            ..Default::default()
                        }),
                    },
                ],
                network: NetMode::Unset.into(),
                security: SecurityMode::Sandbox.into(),
                meta: Some(Meta {
                    args: crate::utils::test::to_vec(vec!["cargo", "build"]),
                    env: vec![],
                    cwd: "/".into(),
                    user: "root".into(),

                    extra_hosts: vec![],
                    proxy_env: None,
                }),
            })
        },
    );
}
