mod ui;
mod app;
mod event;
mod run;
mod utils;

use crate::app::{App, FormState};
use run::run_app;

use futures::StreamExt;
use quinn::{ClientConfig, Endpoint, RecvStream, TransportConfig};
use rustls::client::{ClientConfig as RustlsClientConfig, ServerCertVerified, ServerCertVerifier};
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::time::timeout;
use tracing_subscriber::prelude::*;
use shared::models::chat_models::ChatList;
use shared::server_response::Refresh;

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
    let conn_clone = conn.clone();

    tokio::spawn(async move {
        let recv_stream = conn_clone.accept_uni().await;
        if let Ok(stream) = recv_stream {
            check_for_refresh(stream).await;
        }
    });

    let mut app = App::new(conn);
    run_app(&mut app).await?;
    Ok(())
}

async fn check_for_refresh(mut recv: RecvStream) {
    loop {
        let mut len_buf = [0u8; 4];

        let result = timeout(Duration::from_millis(10), recv.read_exact(&mut len_buf)).await;

        match result {
            Ok(Ok(_)) => {
                let len = u32::from_be_bytes(len_buf) as usize;
                let mut buf = vec![0u8; len];
                if let Ok(_) = recv.read_exact(&mut buf).await {
                    let response: Result<Refresh, _> = serde_json::from_slice(&buf);
                    println!("Received refresh command");
                }
            }
            Ok(Err(e)) => {
                eprintln!("recv error: {:?}", e);
                break;
            }
            Err(_) => {
                // timeout -> no data, keep looping or return
                break;
            }
        }
    }
}