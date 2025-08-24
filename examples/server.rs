use std::sync::Arc;
use radius_server::{
    dictionary::Dictionary,
    handler::serve_accounting_async,
    packet::{AccountingPacket, RadiusAttribute},
    serve_async,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dict = Arc::new(Dictionary::load_embedded()?);
    let secret = "test123";

    let dict_acct = dict.clone();
    let dict_auth = dict.clone();
    let secret_acct = secret.to_string();
    let secret_auth = secret.to_string();

    let acct_server = serve_accounting_async("0.0.0.0:1813", dict_acct, &secret_acct, move |packet| async move {
        println!("📨 Accounting ID {} from {:?}", packet.identifier, packet.username());
        for attr in packet.attributes {
            println!("  → Type {}: {:?}", attr.typ, attr.value);
        }
        Ok(())
    });

    let auth_server = serve_async("0.0.0.0:1812", dict_auth, &secret_auth, move |packet| async move {
        println!("🔍 Incoming ID {} from {:?}", packet.identifier, packet.username());
      let acc_pkt: AccountingPacket = packet.clone().into();
    println!("{:?}", acc_pkt);

        if let Some(username) = packet.username() {
            if username.trim() == " " {
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
    });

    // Run both servers concurrently
    tokio::try_join!(acct_server, auth_server)?;

    Ok(())
}
