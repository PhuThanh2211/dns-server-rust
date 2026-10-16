use crate::header::DnsHeader;
use crate::question::DnsQuestion;
use crate::record::DnsRecord;

pub struct DnsMessage {
    header: DnsHeader,
    questions: Vec<DnsQuestion>,
    answers: Vec<DnsRecord>,
}
impl DnsMessage {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&self.header.to_bytes());

        for q in &self.questions {
            out.extend_from_slice(&q.to_bytes());
        }

        for a in &self.answers {
            out.extend_from_slice(&a.to_bytes());
        }

        out
    }
    pub fn parse(buf: &[u8]) -> Option<DnsMessage> {
        if buf.len() < 12 { return None; }
        let header = DnsHeader::from_bytes(buf);
        let mut pos = 12;
        let mut questions = Vec::new();
        for _ in 0..header.qdcount {
            let (q, next) = DnsQuestion::from_bytes(buf, pos)?;
            questions.push(q);
            pos = next;
        }
        Some(DnsMessage { header, questions, answers: vec![] })
    }

    pub fn build_response(&self) -> DnsMessage {
        let answers: Vec<DnsRecord> = self.questions.iter()
            .map(|q| DnsRecord::a(&q.name, 60, [8, 8, 8, 8]))
            .collect();
        DnsMessage {
            header: DnsHeader {
                id: self.header.id,
                qr: true,
                opcode: self.header.opcode,
                rd: self.header.rd,
                rcode: if self.header.opcode == 0 { 0 } else { 4 },
                qdcount: self.questions.len() as u16,
                ancount: answers.len() as u16,
                ..Default::default()
            },
            questions: self.questions.clone(),
            answers,
        }
    }
}