use std::sync::Arc;
use radius_server::{
    dictionary::Dictionary,
    packet::RadiusAttribute,
    serve_async,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the RADIUS dictionary
    let dict = Arc::new(Dictionary::load_embedded()?);
    let secret = "test123";

    // Start the RADIUS server using an async handler
    serve_async("0.0.0.0:1812", dict, secret, move |packet| async move {
        println!("🔍 Incoming ID {} from {:?}", packet.identifier, packet.username());

        if let Some(username) = packet.username() {
            if username.trim() == "ec:30:b3:6d:24:6a" {
                Ok(packet.reply_accept(vec![
                    RadiusAttribute::session_timeout(3600),
                    RadiusAttribute::reply_message("Welcome, admin."),
                ]))
            } else {
                Ok(packet.reply_reject("User not allowed"))
            }
        } else {
            Ok(packet.reply_reject("Missing username"))
        }
    })
    .await
}
