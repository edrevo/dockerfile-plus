use std::io::{self, Write};
use std::iter::once;

use buildkit_proto::pb::{self, Input};
use prost::Message;

use crate::serialization::{Context, Node, Result};
use crate::utils::OperationOutput;

/// Final operation in the graph. Responsible for printing the complete LLB definition.
#[derive(Debug)]
pub struct Terminal<'a> {
    input: OperationOutput<'a>,
}

impl<'a> Terminal<'a> {
    pub fn with(input: OperationOutput<'a>) -> Self {
        Self { input }
    }

    pub fn into_definition(self) -> pb::Definition {
        let mut cx = Context::default();
        let final_node_iter = once(self.serialize(&mut cx).unwrap());

        let (def, metadata) = {
            cx.into_registered_nodes()
                .chain(final_node_iter)
                .map(|node| (node.bytes, (node.digest, node.metadata)))
                .unzip()
        };

        pb::Definition { def, metadata }
    }

    pub fn write_definition(self, mut writer: impl Write) -> io::Result<()> {
        let mut bytes = Vec::new();
        self.into_definition().encode(&mut bytes).unwrap();

        writer.write_all(&bytes)
    }

    fn serialize(&self, cx: &mut Context) -> Result<Node> {
        let final_op = pb::Op {
            inputs: vec![Input {
                digest: cx.register(self.input.operation())?.digest.clone(),
                index: self.input.output().into(),
            }],

            ..Default::default()
        };

        Ok(Node::new(final_op, Default::default()))
    }
}

#[test]
fn serialization() {
    use crate::prelude::*;

    let context = Source::local("context");
    let builder_image = Source::image("rustlang/rust:nightly");
    let final_image = Source::image("library/alpine:latest");

    let first_command = Command::run("rustc")
        .args(&["--crate-name", "crate-1"])
        .mount(Mount::ReadOnlyLayer(builder_image.output(), "/"))
        .mount(Mount::ReadOnlyLayer(context.output(), "/context"))
        .mount(Mount::Scratch(OutputIdx(0), "/target"));

    let second_command = Command::run("rustc")
        .args(&["--crate-name", "crate-2"])
        .mount(Mount::ReadOnlyLayer(builder_image.output(), "/"))
        .mount(Mount::ReadOnlyLayer(context.output(), "/context"))
        .mount(Mount::Scratch(OutputIdx(0), "/target"));

    let assembly_op = FileSystem::sequence()
        .append(FileSystem::mkdir(
            OutputIdx(0),
            LayerPath::Other(final_image.output(), "/output"),
        ))
        .append(
            FileSystem::copy()
                .from(LayerPath::Other(first_command.output(0), "/target/crate-1"))
                .to(
                    OutputIdx(1),
                    LayerPath::Own(OwnOutputIdx(0), "/output/crate-1"),
                ),
        )
        .append(
            FileSystem::copy()
                .from(LayerPath::Other(
                    second_command.output(0),
                    "/target/crate-2",
                ))
                .to(
                    OutputIdx(2),
                    LayerPath::Own(OwnOutputIdx(1), "/output/crate-2"),
                ),
        );

    let definition = Terminal::with(assembly_op.output(0)).into_definition();

    assert_eq!(
        definition
            .def
            .iter()
            .map(|bytes| Node::get_digest(&bytes))
            .collect::<Vec<_>>(),
        crate::utils::test::to_vec(vec![
            "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702",
            "sha256:dee2a3d7dd482dd8098ba543ff1dcb01efd29fcd16fdb0979ef556f38564543a",
            "sha256:0e6b31ceed3e6dc542018f35a53a0e857e6a188453d32a2a5bbe7aa2971c1220",
            "sha256:782f343f8f4ee33e4f342ed4209ad1a9eb4582485e45251595a5211ebf2b3cbf",
            "sha256:3418ad515958b5e68fd45c9d6fbc8d2ce7d567a956150d22ff529a3fea401aa2",
            "sha256:13bb644e4ec0cabe836392649a04551686e69613b1ea9c89a1a8f3bc86181791",
            "sha256:d13a773a61236be3c7d539f3ef6d583095c32d2a2a60deda86e71705f2dbc99b",
        ])
    );

    let mut metadata_digests = {
        definition
            .metadata
            .iter()
            .map(|(digest, _)| digest.as_str())
            .collect::<Vec<_>>()
    };

    metadata_digests.sort();
    assert_eq!(
        metadata_digests,
        vec![
            "sha256:0e6b31ceed3e6dc542018f35a53a0e857e6a188453d32a2a5bbe7aa2971c1220",
            "sha256:13bb644e4ec0cabe836392649a04551686e69613b1ea9c89a1a8f3bc86181791",
            "sha256:3418ad515958b5e68fd45c9d6fbc8d2ce7d567a956150d22ff529a3fea401aa2",
            "sha256:782f343f8f4ee33e4f342ed4209ad1a9eb4582485e45251595a5211ebf2b3cbf",
            "sha256:a60212791641cbeaa3a49de4f7dff9e40ae50ec19d1be9607232037c1db16702",
            "sha256:d13a773a61236be3c7d539f3ef6d583095c32d2a2a60deda86e71705f2dbc99b",
            "sha256:dee2a3d7dd482dd8098ba543ff1dcb01efd29fcd16fdb0979ef556f38564543a",
        ]
    );
}
