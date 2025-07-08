use std::str;

#[derive(Debug,Clone)]
pub struct RadiusPacket {
    pub code: u8,
    pub identifier: u8,
    pub length: u16,
    pub authenticator: [u8; 16],
    pub attributes: Vec<RadiusAttribute>,
}

#[derive(Debug,Clone)]
pub struct RadiusAttribute {
    pub typ: u8,
    pub len: u8,
    pub value: Vec<u8>,
}

impl RadiusAttribute {
    pub fn reply_message(msg: &str) -> Self {
        let value = msg.as_bytes().to_vec();
        let len = (value.len() + 2) as u8;

        RadiusAttribute {
            typ: 18,
            len,
            value,
        }
    }

    pub fn user_name(name: &str) -> Self {
        let value = name.as_bytes().to_vec();
        let len = (value.len() + 2) as u8;

        RadiusAttribute {
            typ: 1,
            len,
            value,
        }
    }

    pub fn vendor_specific(vendor_id: u32, payload: Vec<u8>) -> Self {
        let mut value = Vec::new();
        value.extend_from_slice(&vendor_id.to_be_bytes());
        value.extend(payload);

        RadiusAttribute {
            typ: 26,
            len: (2 + value.len()) as u8,
            value,
        }
    }

    pub fn wispr_bandwidth_max_up(bps: u32) -> Self {
        let mut value = Vec::new();
        value.extend_from_slice(&14122u32.to_be_bytes());
        value.push(7);
        value.push(6);
        value.extend_from_slice(&bps.to_be_bytes());

        RadiusAttribute {
            typ: 26,
            len: (2 + value.len()) as u8,
            value,
        }
    }

    pub fn wispr_bandwidth_max_down(bps: u32) -> Self {
        let mut value = Vec::new();
        value.extend_from_slice(&14122u32.to_be_bytes());
        value.push(8);
        value.push(6);
        value.extend_from_slice(&bps.to_be_bytes());

        RadiusAttribute {
            typ: 26,
            len: (2 + value.len()) as u8,
            value,
        }
    }

    pub fn session_timeout(seconds: u32) -> Self {
        RadiusAttribute {
            typ: 27,
            len: 6,
            value: seconds.to_be_bytes().to_vec(),
        }
    }

    pub fn idle_timeout(seconds: u32) -> Self {
        RadiusAttribute {
            typ: 28,
            len: 6,
            value: seconds.to_be_bytes().to_vec(),
        }
    }
}

impl RadiusPacket {
    pub fn from_bytes(buf: &[u8]) -> Result<Self, String> {
        if buf.len() < 20 {
            return Err("Packet too short".to_string());
        }

        let code = buf[0];
        let identifier = buf[1];
        let length = u16::from_be_bytes([buf[2], buf[3]]) as usize;

        if buf.len() < length {
            return Err(format!(
                "Length mismatch: header says {}, but got {} bytes",
                length,
                buf.len()
            ));
        }

        let authenticator: [u8; 16] = buf[4..20].try_into().unwrap();
        let mut attributes = Vec::new();
        let mut i = 20;

        while i < length {
            if i + 2 > length {
                return Err("Truncated attribute header".to_string());
            }

            let typ = buf[i];
            let len = buf[i + 1] as usize;

            if len < 2 || i + len > length {
                return Err(format!(
                    "Invalid attribute length at index {}: len = {}",
                    i, len
                ));
            }

            let value = buf[i + 2..i + len].to_vec();
            attributes.push(RadiusAttribute {
                typ,
                len: len as u8,
                value,
            });

            i += len;
        }

        Ok(RadiusPacket {
            code,
            identifier,
            length: length as u16,
            authenticator,
            attributes,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.push(self.code);
        buf.push(self.identifier);
        buf.extend_from_slice(&[0x00, 0x00]);
        buf.extend_from_slice(&self.authenticator);

        for attr in &self.attributes {
            buf.push(attr.typ);
            buf.push(attr.len);
            buf.extend_from_slice(&attr.value);
        }

        let length = buf.len() as u16;
        buf[2] = (length >> 8) as u8;
        buf[3] = length as u8;

        buf
    }

    pub fn access_accept(identifier: u8, attributes: Vec<RadiusAttribute>) -> Self {
        RadiusPacket {
            code: 2,
            identifier,
            length: 0,
            authenticator: [0u8; 16],
            attributes,
        }
    }

    pub fn access_reject(identifier: u8, msg: &str) -> Self {
        RadiusPacket {
            code: 3,
            identifier,
            length: 0,
            authenticator: [0u8; 16],
            attributes: vec![RadiusAttribute::reply_message(msg)],
        }
    }

    pub fn access_challenge(identifier: u8, msg: &str) -> Self {
        RadiusPacket {
            code: 11,
            identifier,
            length: 0,
            authenticator: [0u8; 16],
            attributes: vec![RadiusAttribute::reply_message(msg)],
        }
    }

    pub fn log(&self) {
        let code_name = match self.code {
            1 => "Access-Request",
            2 => "Access-Accept",
            3 => "Access-Reject",
            _ => "Unknown",
        };

        println!("📨 RADIUS {} (id: {})", code_name, self.identifier);

        for attr in &self.attributes {
            let name = radius_type_name(attr.typ);
            if attr.typ == 1 || attr.typ == 18 || attr.typ == 80 {
                match str::from_utf8(&attr.value) {
                    Ok(val) => println!("  • {}: {}", name, val.trim()),
                    Err(_) => println!("  • {}: {:?}", name, attr.value),
                }
            } else if attr.typ == 26 {
                println!("  • {}: {}", name, decode_vendor_specific(&attr.value));
            } else {
                println!("  • {}: {:?}", name, attr.value);
            }
        }
    }
      pub fn reply_accept(&self, attributes: Vec<RadiusAttribute>) -> RadiusPacket {
        RadiusPacket {
            code: 2,
            identifier: self.identifier,
            length: 0,
            authenticator: [0; 16],
            attributes,
        }
    }

    pub fn reply_reject(&self, message: &str) -> RadiusPacket {
        RadiusPacket {
            code: 3,
            identifier: self.identifier,
            length: 0,
            authenticator: [0; 16],
            attributes: vec![RadiusAttribute::reply_message(message)],
        }
    }

    pub fn reply_challenge(&self, message: &str) -> RadiusPacket {
        RadiusPacket {
            code: 11,
            identifier: self.identifier,
            length: 0,
            authenticator: [0; 16],
            attributes: vec![RadiusAttribute::reply_message(message)],
        }
    }

    pub fn username(&self) -> Option<String> {
        self.attributes
            .iter()
            .find(|a| a.typ == 1)
            .and_then(|a| String::from_utf8(a.value.clone()).ok())
    }
}

fn radius_type_name(typ: u8) -> &'static str {
    match typ {
        1 => "User-Name",
        2 => "User-Password",
        3 => "CHAP-Password",
        4 => "NAS-IP-Address",
        18 => "Reply-Message",
        26 => "Vendor-Specific",
        27 => "Session-Timeout",
        28 => "Idle-Timeout",
        80 => "Message-Authenticator",
        _ => "Unknown",
    }
}

fn decode_vendor_specific(value: &[u8]) -> String {
    if value.len() < 6 {
        return "Invalid VSA".to_string();
    }

    let vendor_id = u32::from_be_bytes([value[0], value[1], value[2], value[3]]);
    let vendor_type = value.get(4).copied().unwrap_or(0);
    let vendor_len = value.get(5).copied().unwrap_or(0) as usize;

    let data_start = 6;
    let data_end = data_start + (vendor_len.saturating_sub(2));
    let payload = &value.get(data_start..data_end).unwrap_or(&[]);

    format!("VendorID={}, Type={}, Data={:?}", vendor_id, vendor_type, payload)
}
