const DEFS: &[&str] = &["proto/github.com/moby/buildkit/frontend/gateway/pb/gateway.proto"];
const PATHS: &[&str] = &["proto"];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .compile(DEFS, PATHS)?;

    Ok(())
}
