use mc_fog_uri::FogViewRouterUri;
use mc_fog_view_connection::fog_view_router_client::FogViewRouterGrpcClient;
use std::{str::FromStr, sync::Arc};

fn main() {
    futures::executor::block_on(async_main()).unwrap();
}

async fn async_main() -> Result<(), grpcio::Error> {
    let fog_view_router_uri = FogViewRouterUri::from_str("insecure-fog-view-router://127.0.0.1/")
        .expect("wrong fog view router uri");
    let env = Arc::new(
        grpcio::EnvBuilder::new()
            .name_prefix("Main-RPC".to_string())
            .build(),
    );
    let (logger, _global_logger_guard) =
        mc_common::logger::create_app_logger(mc_common::logger::o!());

    let fog_view_router_client =
        FogViewRouterGrpcClient::new(fog_view_router_uri.clone(), env.clone(), logger.clone());

    fog_view_router_client.request().await
}
