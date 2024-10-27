use fix_engine::{FixField, FixMessage, Tag, msg_type, fix_version};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new FIX message
    let mut msg = FixMessage::new();

    // Add required header fields
    msg.add_field(FixField::new(Tag::BeginString.value(), fix_version::FIX_4_2.to_vec()));
    msg.add_field(FixField::new(Tag::BodyLength.value(), b"100".to_vec()));
    msg.add_field(FixField::new(Tag::MsgType.value(), msg_type::NEW_ORDER_SINGLE.to_vec()));

    // Add additional fields
    msg.add_field(FixField::new(Tag::SenderCompID.value(), b"SENDER".to_vec()));
    msg.add_field(FixField::new(Tag::TargetCompID.value(), b"TARGET".to_vec()));
    msg.add_field(FixField::new(Tag::MsgSeqNum.value(), b"1".to_vec()));
    msg.add_field(FixField::new(Tag::SendingTime.value(), b"20240101-12:00:00.000".to_vec()));

    // Encode the message
    let encoded = msg.encode()?;
    println!("Encoded message: {:?}", String::from_utf8_lossy(&encoded));

    // Decode the message
    let decoded = FixMessage::decode(&encoded)?;
    println!("Decoded BeginString: {:?}",
             String::from_utf8_lossy(decoded.get_field(Tag::BeginString.value()).unwrap().value()));
    println!("Decoded MsgType: {:?}",
             String::from_utf8_lossy(decoded.get_field(Tag::MsgType.value()).unwrap().value()));

    Ok(())
}