mod ui;
mod app;
mod event;
mod run;
mod utils;

use crate::app::App;
use run::run_app;

use futures::StreamExt;
use quinn::{ClientConfig, Endpoint, TransportConfig};
use rustls::client::{ClientConfig as RustlsClientConfig, ServerCertVerified, ServerCertVerifier};
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing_subscriber::prelude::*;

struct TestVerifier;
impl ServerCertVerifier for TestVerifier {
    fn verify_server_cert(
        &self,
        _: &rustls::Certificate,
        _: &[rustls::Certificate],
        _: &rustls::ServerName,
        _: &mut dyn Iterator<Item = &[u8]>,
        _: &[u8],
        _: SystemTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

#[tokio::main]
async fn main()  -> Result<(), Box<dyn Error>>{
    let mut serv_addr = utils::constants::SERVER_ADDR.to_owned();
    // QUIC Client
    let rustls_cfg = RustlsClientConfig::builder().with_safe_defaults().with_custom_certificate_verifier(Arc::new(TestVerifier)).with_no_client_auth();
    let mut client_cfg = ClientConfig::new(Arc::new(rustls_cfg));

    let mut transport_config = TransportConfig::default();
    transport_config.max_idle_timeout(
        Some(Duration::from_secs(300)
            .try_into()
            .expect("valid idle timeout")),
    );
    transport_config.keep_alive_interval(Some(Duration::from_secs(30)));

    client_cfg.transport_config( Arc::new(transport_config) );

    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_cfg);

    let port = format!("{}:{}", serv_addr, "8080");
    let server_addr: SocketAddr = port.parse()?;
    let new_conn = endpoint.connect(server_addr, &*serv_addr)?.await?;
    let conn = Arc::new(new_conn);

    let mut app = App::new();
    run_app(&mut app, conn.clone()).await?;
    Ok(())
}