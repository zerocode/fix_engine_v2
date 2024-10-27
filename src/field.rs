use bytes::{BufMut, BytesMut};
use std::fmt;
use smallvec::SmallVec;
use rustc_hash::FxHashMap;
use itoa::Buffer as ItoaBuffer;

pub const SOH: u8 = 0x01;
pub const EQUALS: u8 = b'=';

thread_local! {
    static TAG_BUFFER: std::cell::RefCell<ItoaBuffer> = std::cell::RefCell::new(ItoaBuffer::new());
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixField {
    tag: u32,
    value: SmallVec<[u8; 32]>, // Most FIX fields are small, optimize for stack allocation
}

impl FixField {
    #[inline]
    pub fn new(tag: u32, value: impl Into<SmallVec<[u8; 32]>>) -> Self {
        Self {
            tag,
            value: value.into()
        }
    }

    #[inline]
    pub fn tag(&self) -> u32 {
        self.tag
    }

    #[inline]
    pub fn value(&self) -> &[u8] {
        &self.value
    }

    #[inline]
    pub fn encode(&self, buf: &mut BytesMut) {
        TAG_BUFFER.with(|buffer| {
            let mut buffer = buffer.borrow_mut();
            buf.put_slice(buffer.format(self.tag).as_bytes());
        });
        buf.put_u8(EQUALS);
        buf.put_slice(&self.value);
        buf.put_u8(SOH);
    }

    #[inline]
    pub fn encoded_len(&self) -> usize {
        // Pre-calculate tag length using itoa
        TAG_BUFFER.with(|buffer| {
            let mut buffer = buffer.borrow_mut();
            buffer.format(self.tag).len()
        }) + 1 + self.value.len() + 1
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