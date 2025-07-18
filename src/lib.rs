pub mod packet;
pub mod dictionary;
pub mod handler;
use std::sync::Arc;
use tokio::net::UdpSocket;
use crate::{dictionary::Dictionary, packet::RadiusPacket, handler::build_response_with_auth};

pub async fn serve_async<F, Fut>(
    addr: &str,
    dict: Arc<Dictionary>,
    secret: &str,
    handler: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(RadiusPacket) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<RadiusPacket, String>> + Send,
{
    use tokio::net::UdpSocket;

    let socket = UdpSocket::bind(addr).await?;
    let mut buf = [0u8; 1024];

    loop {
        let (len, src) = socket.recv_from(&mut buf).await?;
        let req = RadiusPacket::from_bytes(&buf[..len])?;

        let fut = handler(req.clone());
        let response_result = fut.await;

        let response = match response_result {
            Ok(reply_packet) => build_response_with_auth(reply_packet, req.authenticator, secret),
            Err(err) => {
                eprintln!("❌ Error from handler: {err}");
                build_response_with_auth(req.reply_reject("Internal Error"), req.authenticator, secret)
            }
        };

        socket.send_to(&response.to_bytes(), src).await?;
    }
}
