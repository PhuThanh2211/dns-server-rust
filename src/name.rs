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
    let mut end: Option<usize> = None; // where the caller continues reading
    let mut jumps = 0;

    loop {
        let len = *buf.get(pos)?;

        // Pointer: top two bits are 11
        if len & 0xC0 == 0xC0 {
            let next = *buf.get(pos + 1)?;
            if end.is_none() {
                end = Some(pos + 2); // remember: continue after the 2 pointer bytes
            }
            jumps += 1;
            if jumps > 16 {
                return None; // protect against pointer loops
            }
            pos = (((len & 0x3F) as usize) << 8) | next as usize;
            continue;
        }

        // Normal label
        pos += 1;
        if len == 0 {
            break;
        }
        let label = buf.get(pos..pos + len as usize)?;
        labels.push(String::from_utf8_lossy(label).to_string());
        pos += len as usize;
    }

    Some((labels.join("."), end.unwrap_or(pos)))
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

    #[test]
    fn parse_compressed_name() {
        let mut buf = vec![0u8; 12];
        buf.extend(encode_name("abc.longassdomainname.com")); // at 12
        buf.extend([0, 1, 0, 1]);
        let q2 = buf.len();
        buf.extend([3, b'd', b'e', b'f', 0xC0, 16]);           // def + pointer to 16

        assert_eq!(
            parse_name(&buf, q2),
            Some(("def.longassdomainname.com".to_string(), q2 + 6))
        );
    }

    #[test]
    fn pointer_only_name() {
        let mut buf = vec![0u8; 12];
        buf.extend(encode_name("a.io"));
        let p = buf.len();
        buf.extend([0xC0, 12]);
        assert_eq!(parse_name(&buf, p), Some(("a.io".to_string(), p + 2)));
    }

    #[test]
    fn pointer_loop_returns_none() {
        let mut buf = vec![0u8; 12];
        buf.extend([0xC0, 12]); // points to itself
        assert_eq!(parse_name(&buf, 12), None);
    }
}