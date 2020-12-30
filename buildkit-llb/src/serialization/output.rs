use buildkit_proto::pb;
use prost::Message;
use sha2::{Digest, Sha256};

#[derive(Debug, Default, Clone)]
pub(crate) struct Node {
    pub bytes: Vec<u8>,
    pub digest: String,
    pub metadata: pb::OpMetadata,
}

impl Node {
    pub fn new(message: pb::Op, metadata: pb::OpMetadata) -> Self {
        let mut bytes = Vec::new();
        message.encode(&mut bytes).unwrap();

        Self {
            digest: Self::get_digest(&bytes),
            bytes,
            metadata,
        }
    }

    pub fn get_digest(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.input(&bytes);

        format!("sha256:{:x}", hasher.result())
    }
}
