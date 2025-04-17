use futures::StreamExt;
use std::error::Error;
use std::net::Ipv4Addr;
use std::os::fd::AsRawFd;
use std::str::FromStr;
use std::sync::Arc;
use tcp::manager::Manager;
use tokio::io;
use tokio::sync::Mutex;
use tokio_tun::Tun;
use tokio::sync::mpsc;
use tokio_util::codec::{FramedRead, LinesCodec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

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
}