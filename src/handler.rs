use std::sync::Arc;
use crate::{
    dictionary::Dictionary,
    packet::{RadiusAttribute, RadiusPacket},
};
use md5;

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
