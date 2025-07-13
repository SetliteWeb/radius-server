use md5;

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
    data.extend_from_slice(&received_auth); // Use received authenticator instead of packet's
    data.extend_from_slice(&packet[20..]);
    data.extend_from_slice(secret.as_bytes());

    let hash = md5::compute(&data);
    hash.0 == received_auth
}

/// Builds an Accounting-Response packet with correct authenticator
pub fn build_accounting_response(identifier: u8, request_auth: [u8; 16], secret: &str) -> Vec<u8> {
    let mut buf = vec![5, identifier, 0x00, 0x14]; // Code=5 (Accounting-Response), Length=20
    let mut authenticator = [0u8; 16];

    let mut temp = buf.clone();
    temp.extend_from_slice(&request_auth);
    temp.extend_from_slice(secret.as_bytes());

    let hash = md5::compute(&temp);
    authenticator.copy_from_slice(&hash.0);
    buf.extend_from_slice(&authenticator);

    buf
}
