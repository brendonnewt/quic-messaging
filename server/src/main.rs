mod utils;
use quinn::{Endpoint, Incoming};
use tokio::sync::mpsc;
use tokio::task;
use std::net::ToSocketAddrs;

use sea_orm::DatabaseConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    // Establish DB connection
    let db_url = utils::constants::DATABASE_URL.clone();
    let db: DatabaseConnection = sea_orm::Database::connect(&db_url).await?;

    loop {}
    /*
    let addr = "[::]:4433".to_socket_addrs()?.next().unwrap();
    let (endpoint, mut incoming) = make_server_endpoint(addr)?;

    // Shared map of connected clients
    let clients: Clients = Arc::new(DashMap::new());

    println!("Server listening on {}", addr);

    while let Some(connecting) = incoming.next().await {
        let clients = clients.clone();

        task::spawn(async move {
            if let Ok(conn) = connecting.await {
                println!("New connection from {}", conn.remote_address());

                // Spawn a session handler for this client
                if let Err(e) = handle_client(conn, clients).await {
                    eprintln!("Connection ended with error: {:?}", e);
                }
            }
        });
    }

     */

    Ok(())
}
