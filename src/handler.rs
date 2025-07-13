use std::sync::Arc;
use crate::{
    dictionary::Dictionary,
    packet::{RadiusAttribute, RadiusPacket},
};
use md5;
use tokio::net::UdpSocket;

/// Builds a RADIUS response packet with the proper Response Authenticator.
pub fn build_response_with_auth(
    mut packet: RadiusPacket,
    request_authenticator: [u8; 16],
    secret: &str,
) -> RadiusPacket {
    let mut buf = Vec::new();
    buf.push(packet.code);
    buf.push(packet.identifier);
    buf.extend_from_slice(&[0x00, 0x00]); // placeholder for length
    buf.extend_from_slice(&request_authenticator);

    for attr in &packet.attributes {
        buf.push(attr.typ);
        buf.push(attr.len);
        buf.extend_from_slice(&attr.value);
    }

    let length = buf.len() as u16;
    buf[2] = (length >> 8) as u8;
    buf[3] = (length & 0xFF) as u8;

    buf.extend_from_slice(secret.as_bytes());

    let hash = md5::compute(&buf);
    let mut authenticator = [0u8; 16];
    authenticator.copy_from_slice(&hash.0);

    packet.length = length;
    packet.authenticator = authenticator;

    packet
}

/// Handles an incoming RADIUS packet and returns a response packet.
pub fn handle(packet: RadiusPacket, dict: Arc<Dictionary>) -> Result<RadiusPacket, String> {
    println!("🔍 Handling RADIUS packet ID: {}", packet.identifier);

    for attr in &packet.attributes {
        let attr_code = attr.typ as u32;
        match dict.attributes.get(&attr_code) {
            Some(def) => {
                let name = &def.name;
                match std::str::from_utf8(&attr.value) {
                    Ok(s) => println!("→ {}: {}", name, s.trim()),
                    Err(_) => println!("→ {}: {:?}", name, attr.value),
                }
            }
            None => println!("→ Unknown Attribute Type {}: {:?}", attr.typ, attr.value),
        }
    }

    let mut attributes = vec![
        RadiusAttribute::reply_message("Access granted via Rust RADIUS server."),
        RadiusAttribute::session_timeout(3600),
        RadiusAttribute::idle_timeout(300),
        RadiusAttribute::wispr_bandwidth_max_up(512_000),
        RadiusAttribute::wispr_bandwidth_max_down(1_000_000),
    ];

    // Optional: Echo back username if present
    if let Some(user_attr) = packet.attributes.iter().find(|a| a.typ == 1) {
        if let Ok(username) = std::str::from_utf8(&user_attr.value) {
            attributes.push(RadiusAttribute::user_name(username));
        }
    }

    let accept = RadiusPacket::access_accept(packet.identifier, attributes);
    let response = build_response_with_auth(accept, packet.authenticator, "test123");
    Ok(response)
}




/// Verifies an Accounting-Request packet's Request Authenticator
pub fn verify_accounting_request_authenticator(
    packet: &[u8],
    secret: &str,
    received_auth: [u8; 16],
) -> bool {
    if packet.len() < 20 {
        return false;
    }

    let mut data = Vec::from(&packet[0..4]);
data.extend_from_slice(&[0u8; 16]); // ✅ 16 zero bytes as per RFC
    data.extend_from_slice(&packet[20..]);
    data.extend_from_slice(secret.as_bytes());

    let hash = md5::compute(&data);
    hash.0 == received_auth
}

/// Builds an Accounting-Response packet
pub fn build_accounting_response(identifier: u8, request_auth: [u8; 16], secret: &str) -> Vec<u8> {
    let mut buf = vec![5, identifier, 0x00, 0x14]; // Code=5, length=20
    let mut temp = buf.clone();
    temp.extend_from_slice(&request_auth);
    temp.extend_from_slice(secret.as_bytes());

    let hash = md5::compute(&temp);
    buf.extend_from_slice(&hash.0);
    buf
}

/// Serves incoming Accounting-Request packets and responds
pub async fn serve_accounting_async<F, Fut>(
    addr: &str,
    dict: Arc<Dictionary>,
    secret: &str,
    handler: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(RadiusPacket) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), String>> + Send,
{
    let socket = UdpSocket::bind(addr).await?;
    println!("📡 Accounting server listening on {addr}");

    let mut buf = [0u8; 1024];
    loop {
        let (len, src) = socket.recv_from(&mut buf).await?;
        let raw_packet = &buf[..len];

        let packet = match RadiusPacket::from_bytes(raw_packet) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("❌ Failed to parse: {e}");
                continue;
            }
        };

        if !verify_accounting_request_authenticator(raw_packet, secret, packet.authenticator) {
            eprintln!("🚫 Invalid accounting authenticator.");
            continue;
        }

        if let Err(e) = handler(packet.clone()).await {
            eprintln!("⚠️  Handler error: {e}");
        }

        let response = build_accounting_response(packet.identifier, packet.authenticator, secret);
        socket.send_to(&response, src).await?;
    }
}
