pub mod entity;
pub mod handlers;
pub mod utils;

use crate::handlers::controllers::auth_controller;
use quinn::{Endpoint, RecvStream, SendStream};
use sea_orm::DatabaseConnection;
use server::utils::errors::server_error::ServerError;
use shared::client_response::{ClientRequest, Command, ServerResponse};
use std::net::{SocketAddr, ToSocketAddrs};
use std::ops::Deref;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info};
use tracing_subscriber;

const MAX_MESSAGE_SIZE: usize = 65536; // 64 KB

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    println!("DATABASE_URL: {:?}", std::env::var("DATABASE_URL"));
    println!("SECRET: {:?}", std::env::var("SECRET"));

    tracing_subscriber::fmt::init();

    // Establish DB connection
    let db_url = utils::constants::DATABASE_URL.to_string();
    let db: DatabaseConnection = sea_orm::Database::connect(&db_url).await?;
    let db_arc = Arc::new(db);

    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
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
                    loop {
                        let req = match get_client_request(&mut recv).await {
                            Ok(req) => req,
                            Err(e) => {
                                // Respond with error JSON and continue listening
                                let res = ServerResponse {
                                    jwt: None,
                                    success: false,
                                    message: Some(e.to_string()),
                                    data: None,
                                };

                                if let Err(e) = send_response(&mut send, res).await {
                                    eprintln!("Error sending error response: {:?}", e);
                                }

                                continue;
                            }
                        };

                        // Determine ClientRequest and compile proper response
                        let response = match req.command {
                            Command::Register { username, password } => {
                                match auth_controller::register(username, password, db.clone())
                                    .await
                                {
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
                            Command::Login { username, password } => {
                                match auth_controller::login(username, password, db.clone()).await {
                                    Ok(response_model) => ServerResponse {
                                        jwt: Some(response_model.token),
                                        success: true,
                                        message: Some("Logged In".into()),
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
                            other => {
                                // Shouldn't be possible, but covering the case.
                                ServerResponse {
                                    jwt: None,
                                    success: false,
                                    message: Some(
                                        ServerError::RequestInvalid(format!("{:?}", other))
                                            .to_string(),
                                    ),
                                    data: None,
                                }
                            }
                        };

                        // Send the response
                        if let Err(e) = send_response(&mut send, response).await {
                            error!("Error sending response: {:?}", e);
                        }
                    }
                });
            }
        }
        Err(e) => eprintln!("Connection failed: {:?}", e),
    }
}

async fn send_response(
    send: &mut SendStream,
    resp: ServerResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    // Send response
    let bytes = serde_json::to_vec(&resp).expect("Failed to serialize response");
    let len = (bytes.len() as u32).to_be_bytes();
    send.write_all(&len).await?;
    send.write_all(&bytes).await?;
    Ok(())
}

async fn get_client_request(recv: &mut RecvStream) -> Result<ClientRequest, ServerError> {
    // Read the JSON message from the stream
    let mut buf = match receive_msg(recv).await {
        Ok(buf) => buf,
        Err(e) => {
            return Err(e);
        }
    };

    // Deserialize ClientRequest
    deserialize_client_request(&mut buf).await
}

async fn receive_msg(recv: &mut RecvStream) -> Result<Vec<u8>, ServerError> {
    // Read exactly 4 bytes for message length
    let mut len_buf = [0u8; 4];
    if recv.read_exact(&mut len_buf).await.is_err() {
        return Err(ServerError::RequestInvalid(
            "Couldn't read JSON length".to_string(),
        )); // Connection closed or error
    }
    let msg_len = u32::from_be_bytes(len_buf) as usize;

    // Check that message isn't too large (protecting against DDoS
    if msg_len > MAX_MESSAGE_SIZE {
        error!("Received message exceeding max allowed size");
        return Err(ServerError::RequestInvalid(
            "Received message exceeding max allowed size".to_string(),
        ));
    }

    // Read exactly `msg_len` bytes for the message
    let mut buf = vec![0u8; msg_len];
    if recv.read_exact(&mut buf).await.is_err() {
        return Err(ServerError::RequestInvalid(
            "Couldn't read JSON".to_string(),
        )); // Connection closed or error
    }

    Ok(buf)
}

async fn deserialize_client_request(buf: &mut Vec<u8>) -> Result<ClientRequest, ServerError> {
    match serde_json::from_slice(&buf) {
        Ok(r) => Ok(r),
        Err(_) => Err(ServerError::RequestInvalid("Invalid JSON".to_string())),
    }
}
