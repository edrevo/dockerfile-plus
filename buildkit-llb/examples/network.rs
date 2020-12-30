use std::io::stdout;

use buildkit_llb::prelude::*;

fn main() {
    let bitflags_archive = Source::http("https://crates.io/api/v1/crates/bitflags/1.0.4/download")
        .with_file_name("bitflags.tar");

    let alpine = Source::image("library/alpine:latest");
    let bitflags_unpacked = {
        Command::run("/bin/tar")
            .args(&[
                "-xvzC",
                "/out",
                "--strip-components=1",
                "-f",
                "/in/bitflags.tar",
            ])
            .mount(Mount::ReadOnlyLayer(alpine.output(), "/"))
            .mount(Mount::ReadOnlyLayer(bitflags_archive.output(), "/in"))
            .mount(Mount::Scratch(OutputIdx(0), "/out"))
    };

    let env_logger_repo = Source::git("https://github.com/sebasmagri/env_logger.git")
        .with_reference("ebf4829f3c04ce9b6d3e5d59fa8770bb71bffca3");

    let fs = {
        FileSystem::sequence()
            .append(
                FileSystem::copy()
                    .from(LayerPath::Other(bitflags_unpacked.output(0), "/Cargo.toml"))
                    .to(OutputIdx(0), LayerPath::Scratch("/bitflags.toml")),
            )
            .append(
                FileSystem::copy()
                    .from(LayerPath::Other(env_logger_repo.output(), "/Cargo.toml"))
                    .to(
                        OutputIdx(1),
                        LayerPath::Own(OwnOutputIdx(0), "/env_logger.toml"),
                    ),
            )
    };

    Terminal::with(fs.output(1))
        .write_definition(stdout())
        .unwrap()
}
