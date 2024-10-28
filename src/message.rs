use crate::error::FixError;
use crate::field::{FixField, SOH};
use crate::tags::Tag;
use bytes::{BufMut, BytesMut};
use memchr::memchr;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

const BEGIN_STRING_TAG: u32 = Tag::BeginString.value();
const BODY_LENGTH_TAG: u32 = Tag::BodyLength.value();
const MSG_TYPE_TAG: u32 = Tag::MsgType.value();
const CHECKSUM_TAG: u32 = Tag::CheckSum.value();

const TYPICAL_MESSAGE_FIELDS: usize = 16; // Typical FIX message size

#[derive(Debug, Clone)]
pub struct FixMessage {
    fields: FxHashMap<u32, FixField>,
    field_order: SmallVec<[u32; TYPICAL_MESSAGE_FIELDS]>,
}

impl FixMessage {
    #[inline]
    pub fn new() -> Self {
        Self {
            fields: FxHashMap::default(),
            field_order: SmallVec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            fields: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            field_order: SmallVec::with_capacity(capacity),
        }
    }

    #[inline]
    pub fn add_field(&mut self, field: FixField) {
        let tag = field.tag();
        self.field_order.push(tag);
        self.fields.insert(tag, field);
    }

    #[inline]
    pub fn get_field(&self, tag: u32) -> Option<&FixField> {
        self.fields.get(&tag)
    }

    pub fn encode(&self) -> Result<BytesMut, FixError> {
        // Pre-calculate message size
        let estimated_size = self.calculate_message_size()?;
        let mut buf = BytesMut::with_capacity(estimated_size);

        // Encode BeginString
        self.encode_field(BEGIN_STRING_TAG, &mut buf)?;

        // Add body length placeholder - use a small fixed size first
        let body_length_start = buf.len();
        buf.extend_from_slice(b"9=");

        // Create temporary buffer for rest of message
        let mut body_buf = BytesMut::with_capacity(estimated_size - body_length_start);

        // Encode message type and remaining fields to body buffer
        self.encode_field(MSG_TYPE_TAG, &mut body_buf)?;

        // Batch encode remaining fields
        for &tag in &self.field_order {
            if tag != BEGIN_STRING_TAG &&
                tag != BODY_LENGTH_TAG &&
                tag != MSG_TYPE_TAG &&
                tag != CHECKSUM_TAG {
                self.encode_field(tag, &mut body_buf)?;
            }
        }

        // Now we know the actual body length
        let body_length = body_buf.len();

        // Write the actual body length
        buf.extend_from_slice(body_length.to_string().as_bytes());
        buf.put_u8(SOH);

        // Add the body
        buf.extend_from_slice(&body_buf);

        // Calculate and add checksum
        let checksum: u32 = buf.iter().map(|&b| b as u32).sum::<u32>() % 256;
        let mut checksum_buf = [0u8; 7]; // "10=XXX|"
        checksum_buf[0..3].copy_from_slice(b"10=");
        let checksum_str = format!("{:03}", checksum);
        checksum_buf[3..6].copy_from_slice(checksum_str.as_bytes());
        checksum_buf[6] = SOH;
        buf.extend_from_slice(&checksum_buf);

        Ok(buf)
    }

    pub fn decode(data: &[u8]) -> Result<Self, FixError> {
        let mut message = FixMessage::with_capacity(TYPICAL_MESSAGE_FIELDS);
        let mut pos = 0;

        // Fast path for required header fields
        pos = Self::extract_field(data, pos, BEGIN_STRING_TAG, &mut message)?;
        pos = Self::extract_field(data, pos, BODY_LENGTH_TAG, &mut message)?;
        pos = Self::extract_field(data, pos, MSG_TYPE_TAG, &mut message)?;

        // Process remaining fields using memchr for faster delimiter search
        while pos < data.len() {
            if let Some(field_end) = memchr(SOH, &data[pos..]) {
                let field_data = &data[pos..pos + field_end];
                if let Some(equals_pos) = memchr(b'=', field_data) {
                    let tag = unsafe {
                        // SAFETY: We know this is valid UTF-8 numeric data from FIX protocol
                        std::str::from_utf8_unchecked(&field_data[..equals_pos])
                    }.parse::<u32>()
                        .map_err(|_| FixError::InvalidFormat)?;

                    let value = SmallVec::from_slice(&field_data[equals_pos + 1..]);
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

            let received_checksum = unsafe {
                // SAFETY: We know this is valid UTF-8 numeric data from FIX protocol
                std::str::from_utf8_unchecked(checksum_field.value())
            }.parse::<u32>()
                .map_err(|_| FixError::InvalidFormat)?;

            if calculated_checksum != received_checksum {
                return Err(FixError::InvalidChecksum);
            }
        } else {
            return Err(FixError::MissingField(CHECKSUM_TAG));
        }

        Ok(message)
    }

    // Possible to remove these iterations, requires bench
    #[inline]
    fn calculate_message_size(&self) -> Result<usize, FixError> {
        let mut size = 0;

        // Add space for standard fields
        if let Some(begin_string) = self.get_field(BEGIN_STRING_TAG) {
            size += begin_string.encoded_len();
        } else {
            return Err(FixError::MissingField(BEGIN_STRING_TAG));
        }

        // Body length field: "9=XXX|"
        size += 2;  // "9="
        size += 10; // Maximum length for a typical body length number
        size += 1;  // SOH

        // Add remaining fields
        for &tag in &self.field_order {
            if tag != BEGIN_STRING_TAG && tag != BODY_LENGTH_TAG {
                if let Some(field) = self.get_field(tag) {
                    size += field.encoded_len();
                } else {
                    return Err(FixError::MissingField(tag));
                }
            }
        }

        // Checksum field: "10=XXX|"
        size += 7;

        Ok(size)
    }

    #[inline]
    fn encode_field(&self, tag: u32, buf: &mut BytesMut) -> Result<(), FixError> {
        self.get_field(tag)
            .ok_or(FixError::MissingField(tag))
            .map(|field| field.encode(buf))
    }

    #[inline]
    fn extract_field(
        data: &[u8],
        start_pos: usize,
        expected_tag: u32,
        message: &mut FixMessage,
    ) -> Result<usize, FixError> {
        if let Some(field_end) = memchr(SOH, &data[start_pos..]) {
            let field_data = &data[start_pos..start_pos + field_end];
            if let Some(equals_pos) = memchr(b'=', field_data) {
                let tag = unsafe {
                    // SAFETY: We know this is valid UTF-8 numeric data from FIX protocol
                    std::str::from_utf8_unchecked(&field_data[..equals_pos])
                }.parse::<u32>()
                    .map_err(|_| FixError::InvalidFormat)?;

                if tag != expected_tag {
                    return Err(FixError::InvalidFormat);
                }

                let value = SmallVec::from_slice(&field_data[equals_pos + 1..]);
                message.add_field(FixField::new(tag, value));
                Ok(start_pos + field_end + 1)
            } else {
                Err(FixError::InvalidFormat)
            }
        } else {
            Err(FixError::InvalidFormat)
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Returns true if the message has no fields
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Returns an iterator over the field tags in their original order
    #[inline]
    pub fn field_tags(&self) -> impl Iterator<Item = &u32> {
        self.field_order.iter()
    }

    /// Returns the capacity of the internal storage
    #[inline]
    #[cfg(test)]
    pub(crate) fn capacity(&self) -> usize {
        self.field_order.capacity()
    }
}