# radius-server

A lightweight, async RADIUS server library written in Rust. Built for performance, extensibility, and compatibility with FreeRADIUS-style dictionaries.

---

## ✨ Features

- ✅ Parses and builds RADIUS packets
- 📚 Loads FreeRADIUS-style dictionaries
- 🔒 Supports shared secret verification
- ⚙️ Custom packet handlers via closures
- 🧩 Vendor-Specific Attribute (VSA) support
- 🚀 Async and multithreaded using `tokio`

---

## 📦 Usage


```toml
radius-server = "0.2.0"
```rust
use std::sync::Arc;
use radius_server::{
    dictionary::Dictionary,
    packet::RadiusAttribute,
    serve_async,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dict = Arc::new(Dictionary::load_embedded()?);
    let secret = "test123";

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
    }).await
}