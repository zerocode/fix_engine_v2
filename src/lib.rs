
pub mod error;
pub mod field;
pub mod message;
pub mod tags;

pub use tags::{Tag, msg_type, fix_version};
pub use error::FixError;
pub use field::FixField;
pub use message::FixMessage;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tags::{Tag, msg_type, fix_version};
    use crate::field::SOH;

    #[test]
    fn test_basic_message_encoding() {
        let mut msg = FixMessage::new();

        msg.add_field(FixField::new(Tag::BeginString.value(), fix_version::FIX_4_2.to_vec()));
        msg.add_field(FixField::new(Tag::BodyLength.value(), b"100".to_vec()));
        msg.add_field(FixField::new(Tag::MsgType.value(), msg_type::NEW_ORDER_SINGLE.to_vec()));
        msg.add_field(FixField::new(Tag::SenderCompID.value(), b"SENDER".to_vec()));
        msg.add_field(FixField::new(Tag::TargetCompID.value(), b"TARGET".to_vec()));

        let encoded = msg.encode().unwrap();
        let decoded = FixMessage::decode(&encoded).unwrap();

        assert_eq!(decoded.get_field(Tag::BeginString.value()).unwrap().value(), fix_version::FIX_4_2);
        assert_eq!(decoded.get_field(Tag::MsgType.value()).unwrap().value(), msg_type::NEW_ORDER_SINGLE);
        assert_eq!(decoded.get_field(Tag::SenderCompID.value()).unwrap().value(), b"SENDER");
        assert_eq!(decoded.get_field(Tag::TargetCompID.value()).unwrap().value(), b"TARGET");
    }

    #[test]
    fn test_checksum_calculation() {
        let mut msg = FixMessage::new();

        msg.add_field(FixField::new(8, b"FIX.4.2".to_vec()));
        msg.add_field(FixField::new(9, b"100".to_vec()));
        msg.add_field(FixField::new(35, b"D".to_vec()));

        let encoded = msg.encode().unwrap();

        // Calculate checksum manually
        let calculated_checksum: u32 = encoded[..encoded.len() - 7]
            .iter()
            .map(|&b| b as u32)
            .sum::<u32>() % 256;

        let checksum_str = format!("10={:03}\x01", calculated_checksum);
        assert!(String::from_utf8_lossy(&encoded[encoded.len() - 7..]).eq(&checksum_str));
    }

    #[test]
    fn test_body_length_calculation() {
        let mut msg = FixMessage::new();

        msg.add_field(FixField::new(8, b"FIX.4.2".to_vec()));
        msg.add_field(FixField::new(9, b"0".to_vec())); // Will be auto-calculated
        msg.add_field(FixField::new(35, b"D".to_vec()));

        let encoded = msg.encode().unwrap();

        // Convert to string for easier debugging
        let msg_str = String::from_utf8_lossy(&encoded);
        println!("Encoded message: {}", msg_str);

        // Find positions of key fields
        let body_length_tag_pos = encoded.windows(2).position(|w| w == b"9=".as_slice()).unwrap();
        let body_length_end = encoded[body_length_tag_pos..].iter().position(|&b| b == SOH).unwrap() + body_length_tag_pos;
        let checksum_pos = encoded.windows(3).position(|w| w == b"10=".as_slice()).unwrap();

        // Extract body length value
        let body_length_str = String::from_utf8_lossy(&encoded[body_length_tag_pos + 2..body_length_end]);
        let body_length: usize = body_length_str.parse().unwrap();

        // Calculate actual body length (from after body length field's SOH to before checksum)
        let actual_length = checksum_pos - (body_length_end + 1);

        assert_eq!(
            body_length,
            actual_length,
            "Body length mismatch. Message: {}\nFound {} in message but actual length is {}",
            msg_str,
            body_length,
            actual_length
        );
    }

    #[test]
    fn test_complex_message_body_length() {
        let mut msg = FixMessage::new();

        msg.add_field(FixField::new(8, b"FIX.4.2".to_vec()));
        msg.add_field(FixField::new(9, b"0".to_vec())); // Will be auto-calculated
        msg.add_field(FixField::new(35, b"D".to_vec()));
        msg.add_field(FixField::new(49, b"SENDER".to_vec()));
        msg.add_field(FixField::new(56, b"TARGET".to_vec()));
        msg.add_field(FixField::new(34, b"1".to_vec()));
        msg.add_field(FixField::new(52, b"20240101-12:00:00.000".to_vec()));

        let encoded = msg.encode().unwrap();

        // Extract and verify body length
        let body_start = encoded.iter()
            .position(|&b| b == b'9')
            .unwrap();
        let body_end = encoded[body_start..]
            .iter()
            .position(|&b| b == 0x01)
            .unwrap() + body_start;

        let body_length_str = String::from_utf8_lossy(&encoded[body_start + 2..body_end]);
        let body_length: usize = body_length_str.parse().unwrap();

        let checksum_start = encoded.windows(3)
            .position(|w| w == b"10=".as_slice())
            .unwrap();
        let actual_length = checksum_start - (body_end + 1);

        assert_eq!(body_length, actual_length);

        // Decode and verify the message can be read back
        let decoded = FixMessage::decode(&encoded).unwrap();
        assert_eq!(decoded.get_field(49).unwrap().value(), b"SENDER");
        assert_eq!(decoded.get_field(56).unwrap().value(), b"TARGET");
    }

    #[test]
    fn test_invalid_checksum() {
        let mut msg = FixMessage::new();

        msg.add_field(FixField::new(8, b"FIX.4.2".to_vec()));
        msg.add_field(FixField::new(9, b"100".to_vec()));
        msg.add_field(FixField::new(35, b"D".to_vec()));

        let mut encoded = msg.encode().unwrap();

        // Corrupt the message
        encoded[5] = b'X';

        assert!(matches!(
            FixMessage::decode(&encoded),
            Err(FixError::InvalidChecksum)
        ));
    }

    #[test]
    fn test_missing_required_fields() {
        let mut msg = FixMessage::new();

        // Missing BeginString (8)
        msg.add_field(FixField::new(9, b"100".to_vec()));
        msg.add_field(FixField::new(35, b"D".to_vec()));

        assert!(matches!(
            msg.encode(),
            Err(FixError::MissingField(8))
        ));
    }

    #[test]
    fn test_field_order() {
        let mut msg = FixMessage::new();

        // Add fields in random order
        msg.add_field(FixField::new(35, b"D".to_vec()));
        msg.add_field(FixField::new(8, b"FIX.4.2".to_vec()));
        msg.add_field(FixField::new(9, b"100".to_vec()));
        msg.add_field(FixField::new(49, b"SENDER".to_vec()));

        let encoded = msg.encode().unwrap();

        // Verify correct order in encoded message
        let begin_string_pos = encoded.iter().position(|&b| b == b'8').unwrap();
        let body_length_pos = encoded.iter().position(|&b| b == b'9').unwrap();
        let msg_type_pos = encoded.iter().position(|&b| b == b'3').unwrap();

        assert!(begin_string_pos < body_length_pos);
        assert!(body_length_pos < msg_type_pos);
    }

    #[test]
    fn test_message_format() {
        let mut msg = FixMessage::new();

        msg.add_field(FixField::new(8, b"FIX.4.2".to_vec()));
        msg.add_field(FixField::new(9, b"0".to_vec())); // Will be auto-calculated
        msg.add_field(FixField::new(35, b"D".to_vec()));

        let encoded = msg.encode().unwrap();
        let msg_str = String::from_utf8_lossy(&encoded);

        // Message should start with 8=FIX.4.2|
        assert!(msg_str.starts_with("8=FIX.4.2\x01"));

        // Should be followed by 9=n|
        let body_length_start = msg_str.find("9=").unwrap();
        assert!(body_length_start > 0);

        // Should end with checksum
        assert!(msg_str.ends_with("\x01"));
        let checksum_part = &msg_str[msg_str.find("10=").unwrap()..];
        assert_eq!(checksum_part.len(), 7); // "10=nnn|"

        // Decode should succeed
        let decoded = FixMessage::decode(&encoded).unwrap();
        assert_eq!(decoded.get_field(35).unwrap().value(), b"D");
    }
}