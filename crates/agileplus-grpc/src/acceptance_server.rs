//! Serving the acceptance decorator without changing legacy embedders.
use agileplus_proto::agileplus::v1::agile_plus_core_service_server::AgilePlusCoreService;
use std::net::SocketAddr;

#[cfg(not(agileplus_proto_stubs))]
pub async fn serve<T: AgilePlusCoreService>(addr: SocketAddr, service: T) -> Result<(), Box<dyn std::error::Error>> {
    use agileplus_proto::agileplus::v1::agile_plus_core_service_server::AgilePlusCoreServiceServer;
    tonic::transport::Server::builder()
        .add_service(AgilePlusCoreServiceServer::new(service))
        .serve_with_shutdown(addr, shutdown()).await?;
    Ok(())
}

#[cfg(agileplus_proto_stubs)]
pub async fn serve<T: AgilePlusCoreService>(_addr: SocketAddr, _service: T) -> Result<(), Box<dyn std::error::Error>> {
    Err("gRPC network server unavailable: protobuf generation was skipped".into())
}

#[cfg(not(agileplus_proto_stubs))]
async fn shutdown() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.expect("Ctrl+C handler"); };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler").recv().await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
