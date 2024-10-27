use bytes::{BufMut, BytesMut};
use std::fmt;

pub const SOH: u8 = 0x01;
pub const EQUALS: u8 = b'=';

#[derive(Debug, Clone, PartialEq)]
pub struct FixField {
    tag: u32,
    value: Vec<u8>,
}

impl FixField {
    pub fn new(tag: u32, value: Vec<u8>) -> Self {
        Self { tag, value }
    }

    pub fn tag(&self) -> u32 {
        self.tag
    }

    pub fn value(&self) -> &[u8] {
        &self.value
    }

    pub fn encode(&self, buf: &mut BytesMut) {
        buf.put_slice(self.tag.to_string().as_bytes());
        buf.put_u8(EQUALS);
        buf.put_slice(&self.value);
        buf.put_u8(SOH);
    }

    pub fn encoded_len(&self) -> usize {
        // tag length + '=' + value length + SOH
        self.tag.to_string().len() + 1 + self.value.len() + 1
    }
}

impl fmt::Display for FixField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}={}",
            self.tag,
            String::from_utf8_lossy(&self.value)
        )
    }
}