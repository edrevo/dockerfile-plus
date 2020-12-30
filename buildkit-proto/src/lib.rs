#[allow(clippy::all)]

pub mod moby {
    pub mod buildkit {
        pub mod v1 {
            pub mod frontend {
                include!(concat!(env!("OUT_DIR"), "/moby.buildkit.v1.frontend.rs"));
            }

            pub mod apicaps {
                include!(concat!(env!("OUT_DIR"), "/moby.buildkit.v1.apicaps.rs"));
            }

            pub mod types {
                include!(concat!(env!("OUT_DIR"), "/moby.buildkit.v1.types.rs"));
            }
        }
    }
}

pub mod google {
    pub mod rpc {
        include!(concat!(env!("OUT_DIR"), "/google.rpc.rs"));
    }
}

pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/pb.rs"));
}

pub mod fsutil {
    pub mod types {
        include!(concat!(env!("OUT_DIR"), "/fsutil.types.rs"));
    }
}
