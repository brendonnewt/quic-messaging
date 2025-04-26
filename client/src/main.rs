mod ui;
mod app;
mod event;
mod run;

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


    // QUIC Client
    let rustls_cfg = RustlsClientConfig::builder().with_safe_defaults().with_custom_certificate_verifier(Arc::new(TestVerifier)).with_no_client_auth();
    let mut client_cfg = ClientConfig::new(Arc::new(rustls_cfg));

    // now configure the QUIC transport:
    let mut transport_config = TransportConfig::default();
    transport_config.max_idle_timeout(
        Some(Duration::from_secs(300)
            .try_into()
            .expect("valid idle timeout")),
    );
    transport_config.keep_alive_interval(Some(Duration::from_secs(30)));

    // attach it
    client_cfg.transport_config( Arc::new(transport_config) );

    // finally:
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_cfg);

    let server_addr: SocketAddr = "192.168.56.1:8080".parse()?;
    let new_conn = endpoint.connect(server_addr, "192.168.56.1")?.await?;
    let conn = Arc::new(new_conn);

    let mut app = App::new();
    run_app(&mut app, conn.clone()).await?;
    Ok(())
/*
    tokio::spawn(async move {
        packet_listener(tun, manager, closed_flag).await.unwrap();
    });

    stdin_listener(tun_clone, manager_clone, closed_flag_clone).await?;

    Ok(())
}

async fn stdin_listener(
) -> Result<(), Box<dyn Error>> {
    let mut stdin = FramedRead::new(io::stdin(), LinesCodec::new());
    loop {
        // Listen for new commands
        let input = stdin
            .next()
            .await
            .transpose()
            .unwrap()
            .unwrap_or("ERR".to_string());

        // Split the string into three args
        // let mut text: Vec<&str> = input.split('"').collect();
        let whitesplit: Vec<&str> = input.split_whitespace().collect();
        let mut args = [""; 3];

        args[..3.min(whitesplit.len())].copy_from_slice(&whitesplit[..3.min(whitesplit.len())]);

        let whitesplit_end = if whitesplit.len() > 2 {
            whitesplit[2..].join(" ")
        } else {
            "".into()
        };
        args[2] = whitesplit_end.as_str();

        // Get the amount of non-whitespace tokens
        let mut arg_len = args.len();
        for arg in args {
            if arg == "" {
                arg_len -= 1;
            }
        }

        match args[0] {
            // CREATE REPL HERE
            _ => {
                // Catches invalid input
                println!("{}", USAGE_STATEMENT);
            }
        }
    }
}

async fn packet_listener(
    tun: Arc<Tun>,
    manager: Arc<Mutex<Manager>>,
    closed_flag: Arc<Mutex<bool>>,
) -> Result<(), Box<dyn Error>> {
    // Define a buffer of size 1500 bytes (maximum Ethernet frame size without CRC) to store received data.
    let mut buf = [0u8; 1500]; // Unlike tun_tap, tokio_tun strips out the top 3 bytes (checks IP for us itself)

    loop {
        // Check if the program has closed
        if {
            // Get the status of the closed flag and release the lock
            let closed = closed_flag.lock().await;
            *closed
        } {
            return Ok(());
        }
        let nbytes = tun.recv(&mut buf[..]).await?;

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[0..nbytes]) {
            Ok(iph) => {
                // let src = iph.source_addr();
                // let dst = iph.destination_addr();
                // let proto = iph.protocol();

                // match proto {
                //     etherparse::IpNumber::ICMP => {
                //         println!(
                //             "Got a TCMP ping from {} to {}. The TUN interface is probably working.",
                //             src, dst
                //         );
                //         continue;
                //     }
                //     etherparse::IpNumber::TCP => {
                //         println!("Got a TCP packet from {} to {}. The TUN interface is probably working.", src, dst);
                //     }
                //     _ => continue,
                // }

                match etherparse::TcpHeaderSlice::from_slice(&buf[iph.slice().len()..nbytes]) {
                    Ok(tcph) => {
                        // Retrieves start of data in packet
                        let data = iph.slice().len() + tcph.slice().len();

                        let mut manager = manager.lock().await;
                        let data = buf[data..nbytes].to_vec();
                        println!("Packet received");
                        match manager
                            .forward_packet(&mut tun.clone(), iph, tcph, data)
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                println!("Failed to forward packet: {}", e);
                            }
                        };
                    }
                    Err(e) => {
                        eprintln!("An error occurred while parsing TCP packet: {:?}", e);
                    }
                }
            }
            Err(_) => {
                //eprintln!("An error occurred while parsing IP packet: {:?}", e);
            }
        }
    }

 */
}