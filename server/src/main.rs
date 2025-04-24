pub mod utils;
pub mod entity;
pub mod handlers;

use shared::client_response::ClientRequest;
use sea_orm::DatabaseConnection;
use std::net::{SocketAddr, ToSocketAddrs};
use std::ops::Deref;
use std::sync::Arc;
use quinn::Endpoint;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    // Establish DB connection
    let db_url = utils::constants::DATABASE_URL.to_string();
    let db: DatabaseConnection = sea_orm::Database::connect(&db_url).await?;
    let db_arc = Arc::new(db);
    
    //handlers::controllers::auth_controller::register("Brendon".to_string(), "Password".to_string(), db_arc.clone()).await?;
    
    let response = handlers::controllers::auth_controller::login("Brendon".to_string(), "Password".to_string(), db_arc.clone()).await?;
    
    println!("Response: {:?}", response);

    let addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let mut endpoint = Endpoint::server(utils::cert::generate_self_signed_cert(), addr)?;

    info!("Server listening on {}", addr);

    while let Some(conn) = endpoint.accept().await {
        tokio::spawn(handle_connection(conn, db_arc.clone()));
    }

    Ok(())

}

async fn handle_connection(conn: quinn::Connecting, db: Arc<sea_orm::DatabaseConnection>) {
    match conn.await {
        Ok(connection) => {
            info!("New connection from {}", connection.remote_address());

            while let Ok((mut send, mut recv)) = connection.accept_bi().await {
                tokio::spawn(async move {
                    let mut buf = vec![0; 1024];
                    match recv.read(&mut buf).await {
                        Ok(Some(n)) => {
                            let msg = String::from_utf8_lossy(&buf[..n]);
                            info!("Received: {}", msg);

                            // Echo the message back to the client
                            if let Err(e) = send.write_all(msg.as_bytes()).await {
                                eprintln!("Failed to send response: {:?}", e);
                            }
                            let _ = send.finish().await;
                        }
                        Ok(None) => {
                            info!("Client closed the stream");
                        }
                        Err(e) => {
                            eprintln!("Failed to read from stream: {:?}", e);
                        }
                    }
                });
            }

        }
        Err(e) => eprintln!("Connection failed: {:?}", e),
    }
}

