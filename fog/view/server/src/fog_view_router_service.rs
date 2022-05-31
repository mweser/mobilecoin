// Copyright (c) 2018-2022 The MobileCoin Foundation

use std::sync::Arc;
// use crate::config::FogViewRouterConfig;
use futures::{future::try_join_all, FutureExt, SinkExt, TryFutureExt, TryStreamExt};
use grpcio::{DuplexSink, RequestStream, RpcContext, WriteFlags};
use mc_attest_api::attest;
use mc_common::logger::{log, Logger};
use mc_fog_api::{
    view::{FogViewRouterRequest, FogViewRouterResponse},
    view_grpc::FogViewRouterApi,
};
use mc_fog_uri::FogViewStoreUri;
use mc_fog_view_enclave_api::ViewEnclaveProxy;
use mc_util_grpc::{rpc_logger, rpc_permissions_error};
use mc_util_metrics::SVC_COUNTERS;

/// TODO: Make a strategy generic.
#[derive(Clone)]
pub struct FogViewRouterService<E: ViewEnclaveProxy> {
    /// config: FogViewRouterConfig,
    /// server: grpcio::Server,
    enclave: E,
    shards: Vec<Arc<FogViewStoreUri>>,
    logger: Logger,
}

impl<E: ViewEnclaveProxy> FogViewRouterService<E> {
    pub fn new(enclave: E, shards: Vec<FogViewStoreUri>, logger: Logger) -> Self {
        let mut shard_arcs: Vec<Arc<FogViewStoreUri>> = Vec::new();
        for shard in shards {
            let shard_arc = Arc::new(shard);
            shard_arcs.push(shard_arc);
        }
        Self {
            enclave,
            shards: shard_arcs,
            logger,
        }
    }
}

impl<E: ViewEnclaveProxy> FogViewRouterApi for FogViewRouterService<E> {
    fn request(
        &mut self,
        ctx: RpcContext,
        requests: RequestStream<FogViewRouterRequest>,
        responses: DuplexSink<FogViewRouterResponse>,
    ) {
        log::info!(self.logger, "Request received in request fn");
        let _timer = SVC_COUNTERS.req(&ctx);
        mc_common::logger::scoped_global_logger(&rpc_logger(&ctx, &self.logger), |logger| {
            let logger = logger.clone();
            // TODO: Confirm that we don't need to perform the authenticator logic. I think
            // we don't  because of streaming...
            let future = handle_request(
                self.shards.clone(),
                self.enclave.clone(),
                requests,
                responses,
                logger.clone(),
            )
            .map_err(move |err: grpcio::Error| log::error!(&logger, "failed to reply: {}", err))
            // TODO: Do stuff with the error
            .map(|_| ());

            ctx.spawn(future)
        });
    }
}

/// Add looping / worker thread. Shove the request into a threadpool. You can do
/// the request.try_next().await? Each thread pool has a thread that keeps
/// calling try_next()
async fn handle_request<E: ViewEnclaveProxy>(
    shards: Vec<Arc<FogViewStoreUri>>,
    enclave: E,
    mut requests: RequestStream<FogViewRouterRequest>,
    mut responses: DuplexSink<FogViewRouterResponse>,
    logger: Logger,
) -> Result<(), grpcio::Error> {
    while let Some(mut request) = requests.try_next().await? {
        if request.has_auth() {
            // TODO: Potentially put this into a thread.
            match enclave.client_accept(request.take_auth().into()) {
                Ok((enclave_response, _)) => {
                    let mut auth_message = attest::AuthMessage::new();
                    auth_message.set_data(enclave_response.into());

                    let mut response = FogViewRouterResponse::new();
                    response.set_auth(auth_message);
                    responses
                        .send((response.clone(), WriteFlags::default()))
                        .await?;
                }
                Err(client_error) => {
                    log::debug!(
                        &logger,
                        "ViewEnclaveApi::client_accept failed: {}",
                        client_error
                    );
                    let rpc_permissions_error = rpc_permissions_error(
                        "client_auth",
                        format!("Permission denied: {:?}", client_error),
                        &logger,
                    );
                    return responses.fail(rpc_permissions_error).await;
                }
            }
        } else if request.has_query() {
            log::info!(logger, "Request has query");
            // 1. Decode the query
            // 2. Split up according to epochs.
            // 3. Assign each of these epoch splits to a shard.

            // Just do this for now
            // 4. Query each shard passing in the appropriate epoch.
            let _result = route_query(shards.clone(), logger.clone()).await;

            // 5. Combine each shard's response and add to the message to send back.
            let response = FogViewRouterResponse::new();
            responses
                .send((response.clone(), WriteFlags::default()))
                .await?;
        } else {
            // TODO: Throw some sort of error though not sure
            //  that's necessary.
        }
    }

    responses.close().await?;
    Ok(())
}

async fn route_query(
    shards: Vec<Arc<FogViewStoreUri>>,
    logger: Logger,
) -> Result<Vec<i32>, String> {
    let mut futures = Vec::new();
    for (i, shard) in shards.iter().enumerate() {
        let future = contact_shard(i, shard.clone(), logger.clone());
        futures.push(future);
    }

    try_join_all(futures).await
}

async fn contact_shard(
    index: usize,
    shard: Arc<FogViewStoreUri>,
    logger: Logger,
) -> Result<i32, String> {
    log::info!(
        logger,
        "hello world from shard {} at index {}",
        shard,
        index
    );
    Ok(0)
}
