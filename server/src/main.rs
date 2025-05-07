pub mod entity;
pub mod handlers;
pub mod utils;

use crate::handlers::controllers::{auth_controller, chat_controller, user_controller};
use quinn::{Endpoint, RecvStream, SendStream};
use sea_orm::DatabaseConnection;
use serde::Serialize;
use serde_json::json;
use server::utils::errors::server_error::ServerError;
use shared::client_response::{ClientRequest, Command};
use shared::server_response::ServerResponse;
use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs};
use std::ops::Deref;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};
use tracing_subscriber;
use dashmap::DashMap;
use shared::models::server_models::ServerResponseModel;

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

    // List of Logged_In Users
    let logged_in = Arc::new(DashMap::<i32, Arc<Mutex<SendStream>>>::new());

    while let Some(conn) = endpoint.accept().await {
        tokio::spawn(handle_connection(conn, db_arc.clone(), logged_in.clone()));
    }

    Ok(())
}

async fn handle_connection(conn: quinn::Connecting, db: Arc<DatabaseConnection>, logged_in: Arc<DashMap<i32, Arc<Mutex<SendStream>>>>) {
    match conn.await {
        Ok(connection) => {
            info!("New connection from {}", connection.remote_address());
            let send_refresh = match connection.open_uni().await {
                Ok(send) => send,
                Err(err) => {
                    error!("{}", err);
                    return;
                }
            };
            let refresh_stream = Arc::new(Mutex::new(send_refresh));

            while let Ok((mut send, mut recv)) = connection.accept_bi().await {
                let db = db.clone();
                let logged_in = logged_in.clone();
                let refresh_clone = refresh_stream.clone();
                tokio::spawn(async move {
                    // Receive messages from the client and respond to them until the connection closes
                        // Get a ClientRequest JSON
                        let req = match get_client_request(&mut recv).await {
                            Ok(req) => req,
                            Err(ServerError::Disconnected) => {
                                info!("Client closed stream");
                                return;
                            }
                            Err(e) => {
                                println!("Client error: {:?}", e);
                                // Respond with error JSON and continue listening
                                if let Err(e) = send_response(
                                    &mut send,
                                    build_response::<(), ServerError>(Err(e), None, ""),
                                )
                                .await
                                {
                                    eprintln!("Error sending error response, closing...: {:?}", e);
                                    return;
                                }
                                return;
                            }
                        };

                        // Match command and forward message to the appropriate controller
                        let response = handle_command(req, db.clone(), logged_in.clone(), refresh_clone).await;

                        let users: Vec<i32> = logged_in.iter().map(|r| r.key().clone()).collect();
                        info!("List Of Logged In Users: {:?}", users);


                        // Send the response
                        if let Err(e) = send_response(&mut send, response).await {
                            error!("Error sending response, closing...: {:?}", e);
                            return;
                        }
                });
            }
        }
        Err(e) => eprintln!("Connection failed: {:?}", e),
    }
}

/// Matches the ClientRequest command to one recognized by the system
/// and returns a response given by the controller for that command
async fn handle_command(req: ClientRequest, db: Arc<DatabaseConnection>, logged_in: Arc<DashMap<i32, Arc<Mutex<SendStream>>>>, refresh_stream: Arc<Mutex<SendStream>>) -> ServerResponse {
    match req.command {
        Command::Register { username, password } => {
            let result = auth_controller::register(username.clone(), password, db.clone()).await;
            // User is automatically logged in upon registration
            if let Ok(auth) = &result {
                logged_in.insert(auth.user_id, refresh_stream);
            }
            let jwt = result.as_ref().ok().map(|r| r.token.clone());
            build_response(result, jwt, "Registered")

        }

        Command::Login { username, password } => {
            let result = auth_controller::login(username.clone(), password, db.clone()).await;
            if let Ok(auth) = &result {
                logged_in.insert(auth.user_id, refresh_stream);
            }
            let jwt = result.as_ref().ok().map(|r| r.token.clone());
            build_response(result, jwt, "Logged in")
        }

        Command::Logout {username} => {
            let result: Result<_, ServerError> = Ok(ServerResponseModel{success: true});
            let user = user_controller::get_user_by_username(username.clone(), db.clone()).await;
            match user {
                Ok(user) => {
                    if logged_in.remove(&user.id).is_some() {
                        info!("User {} logged out", username);
                    } else {
                        warn!("Logout: user {} was not marked as logged in", username);
                    }
                    build_response(result, req.jwt, "Logged out")
                }
                Err(e) => {
                    build_response::<(), utils::errors::server_error::ServerError>(Err(e), None, "")
                }
            }
        }

        Command::SendFriendRequest {receiver_username} => {
            if let Some(jwt) = req.jwt {
                let user = user_controller::get_user_by_username(receiver_username.clone(), db.clone()).await;
                match user {
                    Ok(user) => {
                        build_response(user_controller::add_friend(jwt.clone(), user.id, db.clone()).await, Some(jwt.clone()), "Friend Request Sent")
                    },
                    Err(e) => {
                        build_response::<(), utils::errors::server_error::ServerError>(Err(e), None, "")
                    }
                }
            } else {
                build_response::<(), ServerError>(Err(ServerError::InvalidToken("No token provided".to_string())), None, "")
            }
        }

        Command::GetFriendRequests {} => {
            let jwt = req.jwt;
            let result = user_controller::get_friend_requests(jwt.clone().unwrap(), db.clone()).await;
            build_response(result, jwt.clone(), "Friend Request List Sent")
        }

        Command::AcceptFriendRequest {sender_id} => {
            let jwt = req.jwt;
            let result = user_controller::accept_friend_request(jwt.clone().unwrap(), sender_id, db.clone()).await;
            build_response(result, jwt.clone(), "Friend Request Accepted")
        }

        Command::DeclineFriendRequest {sender_id} => {
            let jwt = req.jwt;
            let result = user_controller::decline_friend_request(jwt.clone().unwrap(), sender_id, db.clone()).await;
            build_response(result, jwt.clone(), "Friend Request Denied")
        }

        Command::RemoveFriend {friend_id} => {
            let jwt = req.jwt;
            let result = user_controller::remove_friend(jwt.clone().unwrap(), friend_id, db.clone()).await;
            build_response(result, jwt.clone(), "Unfriended")
        }

        
        Command::GetChats => {
            if let Some(jwt) = req.jwt {
                build_response(chat_controller::get_user_chats(jwt, db.clone()).await, None, "Chat List")
            } else {
                build_response::<(), ServerError>(Err(ServerError::InvalidToken("No token provided".to_string())), None, "")
            }
        }
        
        Command::GetChatMessages { chat_id, page, page_size} => {
            if let Some(jwt) = req.jwt {
                build_response(chat_controller::get_chat_messages(jwt, chat_id, page, page_size, db.clone()).await, None, "Chat Messages")
            } else {
                build_response::<(), ServerError>(Err(ServerError::InvalidToken("No token provided".to_string())), None, "")
            }
        }
        
        Command::SendMessage { chat_id, content} => {
            if let Some(jwt) = req.jwt {
                let result = chat_controller::send_message(jwt, chat_id, content, db.clone()).await;

                // If a message is sent, get affected online users and notify them
                if result.is_ok() {
                    let user_ids = chat_controller::get_chat_user_ids(chat_id, db.clone()).await;
                    if let Ok(user_ids) = user_ids {
                        notify_users(user_ids, logged_in.clone()).await;
                    }
                }

                // Send the response back
                build_response(result, None, "Message Sent")
            } else {
                build_response::<(), ServerError>(Err(ServerError::InvalidToken("No token provided".to_string())), None, "")
            }
        }

        Command::GetChatPages { chat_id, page_size } => {
            if let Some(jwt) = req.jwt {
                build_response(chat_controller::get_chat_page_count(jwt, chat_id, page_size, db.clone()).await, None, "Chat Page Count")
            } else {
                build_response::<(), ServerError>(Err(ServerError::InvalidToken("No token provided".to_string())), None, "")
            }
        }

        Command::GetFriends => {
            if let Some(jwt) = req.jwt {
                build_response(user_controller::get_friends(jwt, db.clone()).await, None, "Friends")
            } else {
                build_response::<(), ServerError>(Err(ServerError::InvalidToken("No token provided".to_string())), None, "")
            }
        }

        Command::CreateChat { member_ids, name, is_group } => {
            if let Some(jwt) = req.jwt {
                build_response(chat_controller::create_chat(jwt, name, is_group, member_ids, db.clone()).await, None, "Chat Page Count")
            } else {
                build_response::<(), ServerError>(Err(ServerError::InvalidToken("No token provided".to_string())), None, "")
            }
        }
        
        Command::GetUnreadMessageCount => {
            if let Some(jwt) = req.jwt {
                build_response(chat_controller::get_unread_message_count(jwt, db.clone()).await, None, "Unread Message Count")
            } else {
                build_response::<(), ServerError>(Err(ServerError::InvalidToken("No token provided".to_string())), None, "")
            }
        }
        
        Command::MarkMessagesRead { chat_id } => {
            if let Some(jwt) = req.jwt {
                build_response(chat_controller::mark_messages_read(jwt, chat_id, db.clone()).await, None, "Unread Message Count")
            } else {
                build_response::<(), ServerError>(Err(ServerError::InvalidToken("No token provided".to_string())), None, "")
            }
        }

        other => {
            // Shouldn't be possible, but covering the case.
            build_response::<(), ServerError>(
                Err(ServerError::RequestInvalid(format!("{:?}", other))),
                None,
                "",
            )
        }
    }
}

/// Builds a response based on
/// 1: The result of controller call
/// 2: The type of model returned by the controller
pub fn build_response<T, E>(
    result: Result<T, E>,
    jwt: Option<String>,
    message: &str,
) -> ServerResponse
where
    T: Serialize,
    E: std::fmt::Display,
{
    match result {
        Ok(data) => ServerResponse {
            jwt,
            success: true,
            message: Some(message.to_string()),
            data: Some(json!(data)),
        },
        Err(e) => ServerResponse {
            jwt: None,
            success: false,
            message: Some(e.to_string()),
            data: None,
        },
    }
}

/// Uses the QUIC sending stream to send a ServerResponse
async fn send_response(
    send: &mut SendStream,
    resp: ServerResponse,
) -> Result<(), Box<dyn std::error::Error>> {
    // Send response
    let bytes = serde_json::to_vec(&resp).expect("Failed to serialize response");
    let len = (bytes.len() as u32).to_be_bytes();
    send.write_all(&len).await?;
    send.write_all(&bytes).await?;
    send.finish().await?;
    Ok(())
}

/// Receives a message from the client through the QUIC receive stream and
/// deserializes it into a ClientRequest, or returns a ServerError if
/// anything goes wrong
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

/// Gets the client message as a Vec of bytes
async fn receive_msg(recv: &mut RecvStream) -> Result<Vec<u8>, ServerError> {
    // Read exactly 4 bytes for message length
    let mut len_buf = [0u8; 4];
    match recv.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) => {
            println!("Read error: {:?}", e);
            return if e.to_string().contains("early eof") {
                Err(ServerError::Disconnected)
            } else {
                Err(ServerError::RequestInvalid(
                    "Couldn't read message length".into(),
                ))
            }
        }
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

/// Deserializes a Vec of bytes into a ClientRequest
async fn deserialize_client_request(buf: &mut Vec<u8>) -> Result<ClientRequest, ServerError> {
    match serde_json::from_slice(&buf) {
        Ok(r) => Ok(r),
        Err(e) => {
            println!("{}", e.to_string());
            Err(ServerError::RequestInvalid("Invalid JSON".to_string()))
        },
    }
}

async fn notify_users(user_ids: Vec<i32>, stream_map: Arc<DashMap<i32, Arc<Mutex<SendStream>>>>) {
    for user_id in user_ids {
        if let Some(stream_ref) = stream_map.get_mut(&user_id) {
            let mut stream_lock = stream_ref.lock().await;
            println!("Notifying user {} in stream map", user_id);
            let bytes = serde_json::to_vec(&shared::server_response::Refresh).expect("Failed to serialize refresh ping");
            let len = (bytes.len() as u32).to_be_bytes();
            if let Err(e) = stream_lock.write_all(&len).await {
                eprintln!("Failed to notify user {}: {}", user_id, e);
                stream_map.remove(&user_id);
                continue;
            }
            if let Err(e) = stream_lock.write_all(&bytes).await {
                eprintln!("Failed to notify user {}: {}", user_id, e);
                stream_map.remove(&user_id);
                continue;
            }
        }
    }
}
