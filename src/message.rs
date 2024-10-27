use crate::error::FixError;
use crate::field::{FixField, SOH};
use bytes::{BufMut, BytesMut};
use memchr::memchr;
use std::collections::HashMap;
use crate::tags::Tag;

const BEGIN_STRING_TAG: u32 = Tag::BeginString.value();
const BODY_LENGTH_TAG: u32 = Tag::BodyLength.value();
const MSG_TYPE_TAG: u32 = Tag::MsgType.value();
const CHECKSUM_TAG: u32 = Tag::CheckSum.value();

#[derive(Debug, Clone)]
pub struct FixMessage {
    fields: HashMap<u32, FixField>,
    field_order: Vec<u32>,
}

impl FixMessage {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            field_order: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            fields: HashMap::with_capacity(capacity),
            field_order: Vec::with_capacity(capacity),
        }
    }

    pub fn add_field(&mut self, field: FixField) {
        let tag = field.tag();
        self.field_order.push(tag);
        self.fields.insert(tag, field);
    }

    pub fn get_field(&self, tag: u32) -> Option<&FixField> {
        self.fields.get(&tag)
    }

    pub fn encode(&self) -> Result<BytesMut, FixError> {
        let mut buf = BytesMut::new();

        // Encode BeginString (tag 8)
        self.encode_field(BEGIN_STRING_TAG, &mut buf)?;

        // Temporarily add body length field with a placeholder
        // We'll calculate and update this later
        let body_length_start = buf.len();
        buf.extend_from_slice(b"9=0\x01");

        // Mark where the body begins (after body length field)
        let body_start = buf.len();

        // Encode message type and remaining fields
        self.encode_field(MSG_TYPE_TAG, &mut buf)?;

        // Encode remaining fields except checksum
        for &tag in &self.field_order {
            if tag != BEGIN_STRING_TAG &&
                tag != BODY_LENGTH_TAG &&
                tag != MSG_TYPE_TAG &&
                tag != CHECKSUM_TAG {
                self.encode_field(tag, &mut buf)?;
            }
        }

        // Calculate actual body length (from after 9=n|)
        let body_length = buf.len() - body_start;

        // Create new buffer with correct body length
        let mut final_buf = BytesMut::new();

        // Copy begin string
        final_buf.extend_from_slice(&buf[..body_length_start]);

        // Add body length with correct value
        final_buf.extend_from_slice(b"9=");
        final_buf.extend_from_slice(body_length.to_string().as_bytes());
        final_buf.put_u8(SOH);

        // Copy rest of message
        final_buf.extend_from_slice(&buf[body_start..]);

        // Calculate and add checksum
        let mut checksum: u32 = final_buf.iter().map(|&b| b as u32).sum();
        checksum %= 256;

        final_buf.extend_from_slice(format!("10={:03}", checksum).as_bytes());
        final_buf.put_u8(SOH);

        Ok(final_buf)
    }

    pub fn decode(data: &[u8]) -> Result<Self, FixError> {
        let mut message = FixMessage::new();
        let mut pos = 0;

        // Verify and extract required header fields
        pos = Self::extract_field(data, pos, BEGIN_STRING_TAG, &mut message)?;
        pos = Self::extract_field(data, pos, BODY_LENGTH_TAG, &mut message)?;
        pos = Self::extract_field(data, pos, MSG_TYPE_TAG, &mut message)?;

        // Extract remaining fields
        while pos < data.len() {
            if let Some(field_end) = memchr(SOH, &data[pos..]) {
                let field_data = &data[pos..pos + field_end];
                if let Some(equals_pos) = memchr(b'=', field_data) {
                    let tag = std::str::from_utf8(&field_data[..equals_pos])
                        .map_err(|_| FixError::InvalidFormat)?
                        .parse::<u32>()
                        .map_err(|_| FixError::InvalidFormat)?;

                    let value = field_data[equals_pos + 1..].to_vec();
                    message.add_field(FixField::new(tag, value));
                }
                pos += field_end + 1;
            } else {
                return Err(FixError::InvalidFormat);
            }
        }

        // Verify checksum
        if let Some(checksum_field) = message.get_field(CHECKSUM_TAG) {
            let calculated_checksum: u32 = data[..data.len() - 7]
                .iter()
                .map(|&b| b as u32)
                .sum::<u32>() % 256;

            let received_checksum = std::str::from_utf8(checksum_field.value())
                .map_err(|_| FixError::InvalidFormat)?
                .parse::<u32>()
                .map_err(|_| FixError::InvalidFormat)?;

            if calculated_checksum != received_checksum {
                return Err(FixError::InvalidChecksum);
            }
        } else {
            return Err(FixError::MissingField(CHECKSUM_TAG));
        }

        Ok(message)
    }

    fn encode_field(&self, tag: u32, buf: &mut BytesMut) -> Result<(), FixError> {
        if let Some(field) = self.get_field(tag) {
            field.encode(buf);
            Ok(())
        } else {
            Err(FixError::MissingField(tag))
        }
    }

    fn extract_field(
        data: &[u8],
        start_pos: usize,
        expected_tag: u32,
        message: &mut FixMessage,
    ) -> Result<usize, FixError> {
        if let Some(field_end) = memchr(SOH, &data[start_pos..]) {
            let field_data = &data[start_pos..start_pos + field_end];
            if let Some(equals_pos) = memchr(b'=', field_data) {
                let tag = std::str::from_utf8(&field_data[..equals_pos])
                    .map_err(|_| FixError::InvalidFormat)?
                    .parse::<u32>()
                    .map_err(|_| FixError::InvalidFormat)?;

                if tag != expected_tag {
                    return Err(FixError::InvalidFormat);
                }

                let value = field_data[equals_pos + 1..].to_vec();
                message.add_field(FixField::new(tag, value));
                Ok(start_pos + field_end + 1)
            } else {
                Err(FixError::InvalidFormat)
            }
        } else {
            Err(FixError::InvalidFormat)
        }
    }
}