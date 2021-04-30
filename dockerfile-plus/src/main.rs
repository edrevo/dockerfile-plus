use std::path::PathBuf;

use dockerfile_frontend::DockerfileFrontend;

use anyhow::{Context, Result};
use buildkit_llb::prelude::*;
use buildkit_proto::{
    google::rpc::Status,
    moby::buildkit::v1::frontend::{
        llb_bridge_client::LlbBridgeClient, result::Result as RefResult, FileRange,
        ReadFileRequest, ReturnRequest, SolveRequest,
    },
};
use serde::Deserialize;
use tonic::{transport::Channel, transport::Endpoint};
use tower::service_fn;

use futures::executor;

mod dockerfile_frontend;
mod options;
mod stdio;

async fn read_file<P>(
    client: &mut LlbBridgeClient<Channel>,
    layer: &str,
    path: P,
    range: Option<FileRange>,
) -> Result<Vec<u8>>
where
    P: Into<PathBuf>,
{
    let file_path = path.into().display().to_string();

    let request = ReadFileRequest {
        r#ref: layer.to_string(),
        file_path,
        range,
    };

    let response = client.read_file(request).await?.into_inner().data;

    Ok(response)
}

async fn solve(client: &mut LlbBridgeClient<Channel>, graph: Terminal<'_>) -> Result<String> {
    let solve_request = SolveRequest {
        definition: Some(graph.into_definition()),
        exporter_attr: vec![],
        allow_result_return: true,
        ..Default::default()
    };
    let temp_result = client
        .solve(solve_request)
        .await?
        .into_inner()
        .result
        .unwrap()
        .result
        .unwrap();
    match temp_result {
        RefResult::RefDeprecated(inner) => Ok(inner),
        _ => panic!("Unexpected result"),
    }
}

async fn run(mut client: LlbBridgeClient<Channel>) -> Result<ReturnRequest> {
    let o: DockerfileOptions = options::from_env(std::env::vars())?;
    let dockerfile_path = o
        .filename
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or("Dockerfile");
    let dockerfile_source = Source::local("dockerfile");
    let dockerfile_layer = solve(&mut client, Terminal::with(dockerfile_source.output())).await?;
    let dockerfile_contents =
        String::from_utf8(read_file(&mut client, &dockerfile_layer, dockerfile_path, None).await?)?;
    let dockerfile_frontend = DockerfileFrontend::new(client.clone(), dockerfile_path);
    dockerfile_trap(client.clone(), dockerfile_frontend, dockerfile_contents).await
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let channel = {
        Endpoint::from_static("http://[::]:50051")
            .connect_with_connector(service_fn(stdio::stdio_connector))
            .await
            .unwrap()
    };
    let mut client = LlbBridgeClient::new(channel);
    let result = run(client.clone()).await.unwrap_or_else(|e| ReturnRequest {
        result: None,
        error: Some(Status {
            code: 128,
            message: e.to_string(),
            details: vec![],
        }),
    });
    client.r#return(result).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct DockerfileOptions {
    filename: Option<PathBuf>,
}

const INCLUDE_COMMAND: &str = "INCLUDE+";

async fn dockerfile_trap(
    mut client: LlbBridgeClient<Channel>,
    dockerfile_frontend: DockerfileFrontend,
    dockerfile_contents: String,
) -> Result<ReturnRequest> {
    let mut result: Vec<String> = vec![];
    let context_source = Source::local("context");
    let context_layer = solve(&mut client, Terminal::with(context_source.output())).await?;

    fn replace(
        l   : &    String,
        r   : &mut Vec<String>,
        c   : &mut LlbBridgeClient<Channel>,
        ctx : &    String
    )  -> Result<()> {
        if let Some(file_path) = l.trim().strip_prefix(INCLUDE_COMMAND) {
            let bytes = executor::block_on(read_file(c, &ctx, file_path.trim_start().to_string(), None))
                .with_context(|| format!("Could not read file \"{}\". Remember that the file path is relative to the build context, not the Dockerfile path.", file_path))?;
            //recurse
            for l2 in std::str::from_utf8(&bytes)?.to_string().lines() {
                replace(&l2.to_string(), r, c, &ctx)? ;
            }
        } else {
            r.push(l.to_string());
        }
        Ok(())
    }

    for line in dockerfile_contents.lines() {
        replace(&line.to_string(), &mut result, &mut client, &context_layer)? ;
    }
    let dockerfile_contents = result.join("\n");
    dockerfile_frontend.solve(&dockerfile_contents).await
}
