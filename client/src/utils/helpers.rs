use shared::client_response::{ClientRequest, ServerResponse};

pub async fn send_request(
    conn: &quinn::Connection,
    request: &ClientRequest,
) -> Result<ServerResponse, Box<dyn std::error::Error>> {
    let bytes = serde_json::to_vec(request)?;
    let len = (bytes.len() as u32).to_be_bytes();

    let (mut send, mut recv) = conn.open_bi().await?;
    send.write_all(&len).await?;
    send.write_all(&bytes).await?;
    send.finish().await?;

    let mut len_buf = [0u8; 4];
    recv.read_exact(&mut len_buf).await?;
    let resp_len = u32::from_be_bytes(len_buf) as usize;

    let mut resp_buf = vec![0u8; resp_len];
    recv.read_exact(&mut resp_buf).await?;
    let response = serde_json::from_slice(&resp_buf)?;

    Ok(response)
}
