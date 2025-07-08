mod dictionary;
mod packet;
mod handler;

use crate::dictionary::Dictionary;
use crate::packet::RadiusPacket;
use tokio::net::UdpSocket;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ✅ Load dictionary once
    let dict_path = "dictionaries/dictionary"; // adjust if you use a different entry point
    let dictionary = Dictionary::load_from_file(dict_path)?;
    let dictionary = Arc::new(dictionary); // shareable across threads

    let socket = UdpSocket::bind("0.0.0.0:1812").await?;
    let mut buf = [0u8; 1024];

    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;

        // ⛳️ Parse packet
        let packet = RadiusPacket::from_bytes(&buf[..len])?;

        // ✅ Pass dictionary reference to handler
        let response = handler::handle(packet, Arc::clone(&dictionary))?;

        // 🔁 Send response
        socket.send_to(&response.to_bytes(), addr).await?;
    }
}
