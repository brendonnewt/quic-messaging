pub mod utils;
pub mod entity;
pub mod handlers;

use shared::client_response::{ClientRequest, Command, ServerResponse};
use sea_orm::DatabaseConnection;
use std::net::{SocketAddr, ToSocketAddrs};
use std::ops::Deref;
use std::sync::Arc;
use quinn::Endpoint;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::info;
use tracing_subscriber;
use crate::handlers::controllers::auth_controller;

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
                let db = db.clone();
                tokio::spawn(async move {
                    let mut buf = Vec::new();
                    while let Ok(Some(n)) = recv.read_buf(&mut buf).await {
                        if n == 0 {break;}
                    }

                    // Deserialize ClientRequest
                    let req: ClientRequest = match serde_json::from_slice(&buf) {
                        Ok(r) => r,
                        Err(e) => {
                            let response = ServerResponse {
                                jwt: None,
                                success: false,
                                message: Some(format!("Invalid JSON: {}", e)),
                                data: None,
                            };
                            let _ = send.write_all(response.to_string().as_bytes()).await;
                            let _ = send.finish().await;
                            return;
                        }
                    };

                    // Determine ClientRequest and compile proper response
                    let response = match req.command {
                        Command::Register {username, password} => {
                            match auth_controller::register(username, password, db).await {
                                Ok(response_model) => ServerResponse {
                                    jwt: Some(response_model.token),
                                    success: true,
                                    message: Some("Registered".into()),
                                    data: None,
                                },
                                Err(e) => ServerResponse {
                                    jwt: None,
                                    success: false,
                                    message: Some(e.to_string()),
                                    data: None,
                                },
                            }
                        }
                        Command::Login {username, password} => {
                            match auth_controller::login(username, password, db).await {
                                Ok(response_model) => ServerResponse {
                                    jwt: Some(response_model.token),
                                    success: true,
                                    message: Some("Logged In".into()),
                                    data: None,
                                },
                                Err(e) => ServerResponse{
                                    jwt: None,
                                    success: false,
                                    message: Some(e.to_string()),
                                    data: None,
                                }
                            }
                        }
                        other => {
                            // Shouldn't be possible, but covering the case.
                            ServerResponse {
                                jwt: None,
                                success: false,
                                message: Some(format!("Unsupported Command: {:?}", other)),
                                data: None,
                            }
                        }
                    };

                    // Send response
                    let bytes = serde_json::to_vec(&response).expect("Failed to serialize response");
                    if let Err(e) = send.write_all(&bytes).await {
                        eprintln!("Failed to send response: {}", e);
                    }
                    // Close send_half of bi-directional stream in preparation for new stream
                    let _ = send.finish().await;
                });
            }

        }
        Err(e) => eprintln!("Connection failed: {:?}", e),
    }
}

