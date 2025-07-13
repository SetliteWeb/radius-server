Sure! Here's the **full raw markdown** for your updated `README.md`, with both **Access** and **Accounting** support documented cleanly:

---

````markdown
# radius-server

A lightweight, async RADIUS server library written in Rust. Built for performance, extensibility, and compatibility with FreeRADIUS-style dictionaries.

---

## ✨ Features

- ✅ Parses and builds RADIUS packets
- 📚 Loads FreeRADIUS-style dictionaries
- 🔒 Shared secret authenticator validation
- ⚙️ Custom packet handlers via async closures
- 🧩 Vendor-Specific Attribute (VSA) support
- 📡 Built-in support for **Access** and **Accounting**
- 🚀 Fully async with `tokio`

---

## 📦 Usage

### Add to your `Cargo.toml`:

```toml
radius-server = "0.2.0"
````

---

## 🚀 Access Server Example (Auth)

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
```

---

## 📊 Accounting Server Example

```rust
use std::sync::Arc;
use radius_server::{
    dictionary::Dictionary,
    handler::serve_accounting_async,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dict = Arc::new(Dictionary::load_embedded()?);
    let secret = "test123";

    serve_accounting_async("0.0.0.0:1813", dict, secret, move |packet| async move {
        println!("📨 Accounting ID {} from {:?}", packet.identifier, packet.username());
        for attr in packet.attributes {
            println!("  → Type {}: {:?}", attr.typ, attr.value);
        }
        Ok(())
    }).await
}
```

Accounting-Request packets (Code `4`) are validated against RFC 2866 and automatically responded to with a proper Accounting-Response (Code `5`) including a correct MD5 authenticator.

---

## 📂 Dictionary Support

Supports FreeRADIUS-style dictionaries (e.g., `dictionary`, `dictionary.rfc2865`, `dictionary.vsa`).

You can load:

```rust
Dictionary::load_embedded()?;
// or
Dictionary::from_file("path/to/dictionary")?;
```

---

## 🧪 Testing Tools

You can test with `radclient`:

```bash
echo "User-Name = test, Acct-Status-Type = Start" | radclient -x 127.0.0.1:1813 acct test123
```

Or test authentication:

```bash
echo "User-Name = ec:30:b3:6d:24:6a, User-Password = secret" | radclient -x 127.0.0.1 auth test123
```

---

## 🛠 Status Types You Can Handle

Use `Acct-Status-Type` to differentiate events:

* `Start`
* `Stop`
* `Interim-Update`
* `Accounting-On`
* `Accounting-Off`

---

## 🔌 Architecture

* `serve_async(...)` → Access (Auth) handler
* `serve_accounting_async(...)` → Accounting handler
* Use closures for custom logic
* Runs fully async on `tokio`

---

## 📄 License

MIT License. Contributions welcome.

```

---

Let me know if you'd like to:

- Add a badge section (crates.io, docs.rs, license)
- Add examples for custom dictionaries
- Generate it as an `mdbook` or `docs.rs` doc

Happy to help.
```
