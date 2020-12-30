use std::{process::Stdio, sync::Arc};

use crate::stdio::StdioSocket;
use anyhow::Result;
use buildkit_proto::moby::buildkit::v1::frontend::{
    self, llb_bridge_client::LlbBridgeClient, llb_bridge_server::LlbBridge,
};
use crossbeam::{channel, Sender};
use frontend::{llb_bridge_server::LlbBridgeServer, ReadFileResponse};
use tokio::sync::RwLock;
use tonic::{transport::Channel, transport::Server, Request, Response};

pub struct DockerfileFrontend {
    client: LlbBridgeClient<Channel>,
    dockerfile_name: String,
}

impl DockerfileFrontend {
    pub fn new(client: LlbBridgeClient<Channel>, dockerfile_name: &str) -> DockerfileFrontend {
        DockerfileFrontend {
            client,
            dockerfile_name: dockerfile_name.to_string(),
        }
    }

    pub async fn solve(&self, dockerfile_contents: &str) -> Result<frontend::ReturnRequest> {
        let mut dockerfile_front = std::process::Command::new("/bin/dockerfile-frontend")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .envs(std::env::vars())
            .spawn()?;

        let (tx, rx) = channel::bounded(1);
        Server::builder()
            .add_service(LlbBridgeServer::new(ProxyLlbServer::new(
                self.client.clone(),
                tx,
                self.dockerfile_name.clone(),
                dockerfile_contents.as_bytes().to_vec(),
            )))
            .serve_with_incoming(tokio::stream::once(StdioSocket::try_new_rw(
                dockerfile_front.stdout.take().unwrap(),
                dockerfile_front.stdin.take().unwrap(),
            )))
            .await?;

        dockerfile_front.wait()?;
        Ok(rx.recv()?)
    }
}

struct ProxyLlbServer {
    client: Arc<RwLock<LlbBridgeClient<Channel>>>,
    result_sender: Sender<frontend::ReturnRequest>,

    dockerfile_name: String,
    dockerfile_contents: Vec<u8>,
}

impl ProxyLlbServer {
    fn new(
        client: LlbBridgeClient<Channel>,
        result_sender: Sender<frontend::ReturnRequest>,
        dockerfile_name: String,
        dockerfile_contents: Vec<u8>,
    ) -> Self {
        ProxyLlbServer {
            client: Arc::new(RwLock::new(client)),
            result_sender,
            dockerfile_name,
            dockerfile_contents,
        }
    }
}

#[tonic::async_trait]
impl LlbBridge for ProxyLlbServer {
    async fn resolve_image_config(
        &self,
        request: Request<frontend::ResolveImageConfigRequest>,
    ) -> Result<Response<frontend::ResolveImageConfigResponse>, tonic::Status> {
        eprintln!("Resolve image config: {:?}", request);
        let result = self
            .client
            .write()
            .await
            .resolve_image_config(request)
            .await;
        eprintln!("{:?}", result);
        result
    }

    async fn solve(
        &self,
        request: Request<frontend::SolveRequest>,
    ) -> Result<Response<frontend::SolveResponse>, tonic::Status> {
        eprintln!("Solve: {:?}", request);
        let result = self.client.write().await.solve(request).await;
        eprintln!("{:?}", result);
        result
    }

    async fn read_file(
        &self,
        request: Request<frontend::ReadFileRequest>,
    ) -> Result<Response<frontend::ReadFileResponse>, tonic::Status> {
        eprintln!("Read file: {:?}", request);
        let inner = request.into_inner();
        let request = Request::new(inner.clone());
        let result = if inner.file_path == self.dockerfile_name {
            eprintln!("ITS A TRAP!");
            eprintln!(
                "{}",
                std::str::from_utf8(&self.dockerfile_contents).unwrap()
            );
            Ok(Response::new(ReadFileResponse {
                data: self.dockerfile_contents.clone(),
            }))
        } else {
            self.client.write().await.read_file(request).await
        };
        eprintln!("{:?}", result);
        result
    }

    async fn read_dir(
        &self,
        request: Request<frontend::ReadDirRequest>,
    ) -> Result<Response<frontend::ReadDirResponse>, tonic::Status> {
        eprintln!("Read dir: {:?}", request);
        let result = self.client.write().await.read_dir(request).await;
        eprintln!("{:?}", result);
        result
    }

    async fn stat_file(
        &self,
        request: Request<frontend::StatFileRequest>,
    ) -> Result<Response<frontend::StatFileResponse>, tonic::Status> {
        eprintln!("Stat file: {:?}", request);
        let result = self.client.write().await.stat_file(request).await;
        eprintln!("{:?}", result);
        result
    }

    async fn ping(
        &self,
        request: Request<frontend::PingRequest>,
    ) -> Result<Response<frontend::PongResponse>, tonic::Status> {
        eprintln!("Ping: {:?}", request);
        let result = self.client.write().await.ping(request).await;
        eprintln!("{:?}", result);
        result
    }

    async fn r#return(
        &self,
        request: Request<frontend::ReturnRequest>,
    ) -> Result<Response<frontend::ReturnResponse>, tonic::Status> {
        // Do not send return request to buildkit
        let inner = request.into_inner();
        self.result_sender.send(inner).unwrap();
        Ok(Response::new(frontend::ReturnResponse {}))
    }

    async fn inputs(
        &self,
        request: Request<frontend::InputsRequest>,
    ) -> Result<Response<frontend::InputsResponse>, tonic::Status> {
        eprintln!("Inputs: {:?}", request);
        let result = self.client.write().await.inputs(request).await;
        eprintln!("{:?}", result);
        result
    }
}
