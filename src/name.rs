pub fn encode_name(name: &str) -> Vec<u8> {
    let mut out = Vec::new();
    for label in name.split('.') {
        out.push(label.len() as u8);            // <length>
        out.extend_from_slice(label.as_bytes());// <content>
    }
    out.push(0);
    out
}

pub fn parse_name(buf: &[u8], mut pos: usize) -> Option<(String, usize)> {
    let mut labels: Vec<String> = Vec::new();
    loop {
        let len = *buf.get(pos)? as usize;
        pos += 1;
        if len == 0 {
            break;
        }

        let label = buf.get(pos..pos + len)?;
        labels.push(String::from_utf8_lossy(label).to_string());
        pos += len;
    }

    Some((labels.join("."), pos))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_google() {
        assert_eq!(encode_name("google.com"),
                   [6,b'g',b'o',b'o',b'g',b'l',b'e',3,b'c',b'o',b'm',0]);
    }

    #[test]
    fn parse_google_at_offset() {
        let mut buf = vec![0u8; 12];
        buf.extend(encode_name("google.com"));
        assert_eq!(parse_name(&buf, 12), Some(("google.com".into(), 24)));
    }

    #[test]
    fn parse_truncated_returns_none() {
        assert_eq!(parse_name(&[6, b'g', b'o'], 0), None);
    }
}