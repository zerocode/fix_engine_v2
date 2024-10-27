#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tag {
    BeginString = 8,
    BodyLength = 9,
    CheckSum = 10,
    MsgType = 35,
    MsgSeqNum = 34,
    SenderCompID = 49,
    TargetCompID = 56,
    SendingTime = 52,
    // Add other tags as needed
}

impl Tag {
    pub const fn value(&self) -> u32 {
        *self as u32
    }
}

// Message type values
pub mod msg_type {
    pub const HEARTBEAT: &[u8] = b"0";
    pub const TEST_REQUEST: &[u8] = b"1";
    pub const RESEND_REQUEST: &[u8] = b"2";
    pub const REJECT: &[u8] = b"3";
    pub const SEQUENCE_RESET: &[u8] = b"4";
    pub const LOGOUT: &[u8] = b"5";
    pub const LOGON: &[u8] = b"A";
    pub const NEW_ORDER_SINGLE: &[u8] = b"D";
    pub const EXECUTION_REPORT: &[u8] = b"8";
    // Add other message types as needed
}

// FIX versions
pub mod fix_version {
    pub const FIX_4_0: &[u8] = b"FIX.4.0";
    pub const FIX_4_1: &[u8] = b"FIX.4.1";
    pub const FIX_4_2: &[u8] = b"FIX.4.2";
    pub const FIX_4_3: &[u8] = b"FIX.4.3";
    pub const FIX_4_4: &[u8] = b"FIX.4.4";
    pub const FIX_5_0: &[u8] = b"FIX.5.0";
}