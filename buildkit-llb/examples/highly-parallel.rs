use std::io::stdout;

use buildkit_llb::ops::source::ImageSource;
use buildkit_llb::prelude::*;

fn main() {
    let image = Source::image("library/alpine:latest");
    let commands = build_init_commands(&image);
    let commands = build_modify_commands(&image, commands);

    let base_fs = FileSystem::sequence()
        .custom_name("assemble outputs")
        .append(FileSystem::mkdir(
            OutputIdx(0),
            LayerPath::Scratch("/files"),
        ));

    let (final_fs, final_output) =
        commands
            .into_iter()
            .zip(0..)
            .fold((base_fs, 0), |(fs, last_output), (output, idx)| {
                let layer = fs.append(
                    FileSystem::copy()
                        .from(LayerPath::Other(output, format!("/file-{}.out", idx)))
                        .to(
                            OutputIdx(idx + 1),
                            LayerPath::Own(
                                OwnOutputIdx(last_output),
                                format!("/files/file-{}.out", idx),
                            ),
                        ),
                );

                (layer, idx + 1)
            });

    Terminal::with(final_fs.output(final_output))
        .write_definition(stdout())
        .unwrap()
}

fn build_init_commands(image: &ImageSource) -> Vec<OperationOutput> {
    (0..100)
        .map(|idx| {
            let base_dir = format!("/file/{}", idx);
            let shell = format!("echo 'test {}' > /out{}/file.out", idx, base_dir);

            let output_mount = FileSystem::mkdir(OutputIdx(0), LayerPath::Scratch(&base_dir))
                .make_parents(true)
                .into_operation()
                .ignore_cache(true)
                .ref_counted();

            Command::run("/bin/sh")
                .args(&["-c", &shell])
                .mount(Mount::ReadOnlyLayer(image.output(), "/"))
                .mount(Mount::Layer(OutputIdx(0), output_mount.output(0), "/out"))
                .ignore_cache(true)
                .ref_counted()
                .output(0)
        })
        .collect()
}

fn build_modify_commands<'a>(
    image: &'a ImageSource,
    layers: Vec<OperationOutput<'a>>,
) -> Vec<OperationOutput<'a>> {
    layers
        .into_iter()
        .zip(0..)
        .map(|(output, idx)| {
            let shell = format!(
                "sed s/test/modified/ < /in/file/{}/file.in > /out/file-{}.out",
                idx, idx
            );

            Command::run("/bin/sh")
                .args(&["-c", &shell])
                .mount(Mount::ReadOnlyLayer(image.output(), "/"))
                .mount(Mount::Scratch(OutputIdx(0), "/out"))
                .mount(Mount::ReadOnlySelector(
                    output,
                    format!("/in/file/{}/file.in", idx),
                    format!("file/{}/file.out", idx),
                ))
                .ignore_cache(true)
                .ref_counted()
                .output(0)
        })
        .collect()
}
