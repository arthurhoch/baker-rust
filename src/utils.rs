use std::error::Error;

pub fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

pub fn hex_decode(input: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    let bytes = input.as_bytes();
    if bytes.len() % 2 != 0 {
        return Err("Invalid hex string".into());
    }
    let mut out = Vec::with_capacity(bytes.len() / 2);
    let mut i = 0;
    while i < bytes.len() {
        let hi = from_hex(bytes[i])?;
        let lo = from_hex(bytes[i + 1])?;
        out.push((hi << 4) | lo);
        i += 2;
    }
    Ok(out)
}

fn from_hex(byte: u8) -> Result<u8, Box<dyn Error>> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        b'A'..=b'F' => Ok(byte - b'A' + 10),
        _ => Err("Invalid hex character".into()),
    }
}

pub fn is_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip() {
        let data = b"hello world";
        let hex = hex_encode(data);
        let decoded = hex_decode(&hex).unwrap();
        assert_eq!(decoded, data);
    }

    #[test]
    fn invalid_hex_errors() {
        assert!(hex_decode("abc").is_err());
        assert!(hex_decode("zz").is_err());
    }

    #[test]
    fn detects_urls() {
        assert!(is_url("http://example.com/x"));
        assert!(is_url("https://example.com/x"));
        assert!(!is_url("ftp://example.com/x"));
        assert!(!is_url("/tmp/file"));
    }
}
