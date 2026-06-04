#![forbid(unsafe_code)]

//! Bridge pattern for connecting heterogeneous ternary systems.
//!
//! In a tri-axial ternary fleet, different subsystems speak different protocols,
//! use different wire formats, and store data in different memory models. This
//! crate provides the `Bridge` trait and concrete bridge implementations:
//! `ProtocolBridge`, `CodecBridge`, `MemoryBridge`, and `SkillBridge` — the
//! interop layer that makes the fleet talk to each other.

use std::collections::HashMap;

// ── TernaryValue ───────────────────────────────────────────────────────────

/// A ternary value: Negative, Zero, or Positive.
/// Maps to -1, 0, +1 in balanced ternary arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TernaryValue {
    Negative,
    Zero,
    Positive,
}

impl TernaryValue {
    pub fn to_i8(self) -> i8 {
        match self {
            TernaryValue::Negative => -1,
            TernaryValue::Zero => 0,
            TernaryValue::Positive => 1,
        }
    }

    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(TernaryValue::Negative),
            0 => Some(TernaryValue::Zero),
            1 => Some(TernaryValue::Positive),
            _ => None,
        }
    }
}

// ── Bridge Trait ───────────────────────────────────────────────────────────

/// A bridge translates between two representations.
///
/// `A` is the source representation, `B` is the target.
/// Translations can fail if the source has no valid target mapping.
pub trait Bridge<A, B> {
    /// The error type for failed translations.
    type Error;

    /// Translate from A to B.
    fn forward(&self, value: &A) -> Result<B, Self::Error>;

    /// Translate from B to A.
    fn backward(&self, value: &B) -> Result<A, Self::Error>;
}

// ── BridgeError ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeError {
    pub message: String,
}

impl BridgeError {
    pub fn new(msg: &str) -> Self {
        Self { message: msg.to_string() }
    }
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BridgeError: {}", self.message)
    }
}

// ── WireFormat ─────────────────────────────────────────────────────────────

/// Different wire formats for encoding ternary data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WireFormat {
    /// Each ternary digit as one byte: -1 → 0xFF, 0 → 0x00, +1 → 0x01.
    BytePerTrit,
    /// Two trits per byte (packed nibble format).
    Packed,
    /// UTF-8 string representation: "T", "0", "1" (negative="T", zero="0", positive="1").
    Text,
}

// ── ProtocolMessage ────────────────────────────────────────────────────────

/// A message in the ternary-protocol format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolMessage {
    pub source: String,
    pub destination: String,
    pub payload: Vec<TernaryValue>,
    pub sequence: u32,
}

impl ProtocolMessage {
    pub fn new(source: &str, destination: &str, payload: Vec<TernaryValue>) -> Self {
        Self {
            source: source.to_string(),
            destination: destination.to_string(),
            payload,
            sequence: 0,
        }
    }
}

// ── OracleMessage ──────────────────────────────────────────────────────────

/// A message in the Oracle1 I2I format (simulated).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OracleMessage {
    pub from_node: String,
    pub to_node: String,
    pub data: Vec<i8>,
    pub msg_id: u32,
}

impl OracleMessage {
    pub fn new(from: &str, to: &str, data: Vec<i8>) -> Self {
        Self {
            from_node: from.to_string(),
            to_node: to.to_string(),
            data,
            msg_id: 0,
        }
    }
}

// ── ProtocolBridge ─────────────────────────────────────────────────────────

/// Translates between ternary-protocol messages and Oracle1 I2I messages.
pub struct ProtocolBridge {
    next_sequence: std::cell::Cell<u32>,
}

impl ProtocolBridge {
    pub fn new() -> Self {
        Self {
            next_sequence: std::cell::Cell::new(1),
        }
    }

    fn alloc_sequence(&self) -> u32 {
        let seq = self.next_sequence.get();
        self.next_sequence.set(seq + 1);
        seq
    }
}

impl Default for ProtocolBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl Bridge<ProtocolMessage, OracleMessage> for ProtocolBridge {
    type Error = BridgeError;

    fn forward(&self, msg: &ProtocolMessage) -> Result<OracleMessage, Self::Error> {
        let data: Vec<i8> = msg.payload.iter().map(|t| t.to_i8()).collect();
        let mut oracle = OracleMessage::new(&msg.source, &msg.destination, data);
        oracle.msg_id = self.alloc_sequence();
        Ok(oracle)
    }

    fn backward(&self, msg: &OracleMessage) -> Result<ProtocolMessage, Self::Error> {
        let payload: Result<Vec<TernaryValue>, _> = msg.data.iter()
            .map(|&v| TernaryValue::from_i8(v).ok_or_else(|| BridgeError::new(&format!("Invalid ternary value: {}", v))))
            .collect();
        let mut proto = ProtocolMessage::new(&msg.from_node, &msg.to_node, payload?);
        proto.sequence = msg.msg_id;
        Ok(proto)
    }
}

// ── CodecBridge ────────────────────────────────────────────────────────────

/// Encodes and decodes ternary data between wire formats.
pub struct CodecBridge;

impl CodecBridge {
    /// Encode ternary values to the specified wire format.
    pub fn encode(values: &[TernaryValue], format: WireFormat) -> Result<Vec<u8>, BridgeError> {
        match format {
            WireFormat::BytePerTrit => {
                Ok(values.iter().map(|v| v.to_i8() as u8).collect())
            }
            WireFormat::Packed => {
                if values.len() % 2 != 0 {
                    return Err(BridgeError::new("Packed format requires even number of trits"));
                }
                let mut result = Vec::new();
                for chunk in values.chunks(2) {
                    // Map: Negative(-1)->3, Zero(0)->0, Positive(1)->1
                    let high = match chunk[0] {
                        TernaryValue::Negative => 3u8,
                        TernaryValue::Zero => 0u8,
                        TernaryValue::Positive => 1u8,
                    };
                    let low = match chunk[1] {
                        TernaryValue::Negative => 3u8,
                        TernaryValue::Zero => 0u8,
                        TernaryValue::Positive => 1u8,
                    };
                    result.push((high << 4) | low);
                }
                Ok(result)
            }
            WireFormat::Text => {
                let s: String = values.iter().map(|v| match v {
                    TernaryValue::Negative => 'T',
                    TernaryValue::Zero => '0',
                    TernaryValue::Positive => '1',
                }).collect();
                Ok(s.into_bytes())
            }
        }
    }

    fn nibble_to_trit(n: u8) -> Result<TernaryValue, BridgeError> {
        match n {
            0 => Ok(TernaryValue::Zero),
            1 => Ok(TernaryValue::Positive),
            3 => Ok(TernaryValue::Negative),
            _ => Err(BridgeError::new(&format!("Invalid packed trit nibble: {}", n))),
        }
    }

    /// Decode bytes from the specified wire format into ternary values.
    pub fn decode(bytes: &[u8], format: WireFormat) -> Result<Vec<TernaryValue>, BridgeError> {
        match format {
            WireFormat::BytePerTrit => {
                bytes.iter()
                    .map(|&b| {
                        let signed = b as i8;
                        TernaryValue::from_i8(signed)
                            .ok_or_else(|| BridgeError::new(&format!("Invalid trit byte: {}", signed)))
                    })
                    .collect()
            }
            WireFormat::Packed => {
                let mut result = Vec::new();
                for &byte in bytes {
                    let high = (byte >> 4) & 0x0F;
                    let low = byte & 0x0F;
                    result.push(Self::nibble_to_trit(high)?);
                    result.push(Self::nibble_to_trit(low)?);
                }
                Ok(result)
            }
            WireFormat::Text => {
                bytes.iter()
                    .map(|&b| match b as char {
                        'T' => Ok(TernaryValue::Negative),
                        '0' => Ok(TernaryValue::Zero),
                        '1' => Ok(TernaryValue::Positive),
                        c => Err(BridgeError::new(&format!("Invalid text trit: '{}'", c))),
                    })
                    .collect()
            }
        }
    }
}

impl Bridge<Vec<TernaryValue>, Vec<u8>> for CodecBridge {
    type Error = BridgeError;

    fn forward(&self, values: &Vec<TernaryValue>) -> Result<Vec<u8>, Self::Error> {
        Self::encode(values, WireFormat::BytePerTrit)
    }

    fn backward(&self, bytes: &Vec<u8>) -> Result<Vec<TernaryValue>, Self::Error> {
        Self::decode(bytes, WireFormat::BytePerTrit)
    }
}

// ── MemoryBridge ───────────────────────────────────────────────────────────

/// Short-term memory entry: simple key-value with a creation tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StmEntry {
    pub key: String,
    pub value: Vec<TernaryValue>,
    pub created_at: u64,
}

/// Long-term memory entry: key-value with an access count and weight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LtmEntry {
    pub key: String,
    pub value: Vec<TernaryValue>,
    pub access_count: u32,
    pub weight: u32,
}

/// Translates between short-term and long-term memory representations.
/// STM entries are ephemeral; when promoted to LTM, they gain access tracking.
pub struct MemoryBridge;

impl Bridge<StmEntry, LtmEntry> for MemoryBridge {
    type Error = BridgeError;

    fn forward(&self, stm: &StmEntry) -> Result<LtmEntry, Self::Error> {
        Ok(LtmEntry {
            key: stm.key.clone(),
            value: stm.value.clone(),
            access_count: 0,
            weight: 1,
        })
    }

    fn backward(&self, ltm: &LtmEntry) -> Result<StmEntry, Self::Error> {
        Ok(StmEntry {
            key: ltm.key.clone(),
            value: ltm.value.clone(),
            created_at: 0,
        })
    }
}

// ── SkillBridge ────────────────────────────────────────────────────────────

/// A skill descriptor in TypeScript/Equipment format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsSkill {
    pub name: String,
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    pub source: String,
}

/// A skill descriptor in Rust/Construct format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustSkill {
    pub name: String,
    pub params: Vec<String>,
    pub returns: Vec<String>,
    pub module_path: String,
}

/// Translates skill descriptors between TypeScript (Equipment) and Rust (Construct) formats.
pub struct SkillBridge;

impl Bridge<TsSkill, RustSkill> for SkillBridge {
    type Error = BridgeError;

    fn forward(&self, ts: &TsSkill) -> Result<RustSkill, Self::Error> {
        Ok(RustSkill {
            name: ts.name.clone(),
            params: ts.inputs.clone(),
            returns: ts.outputs.clone(),
            module_path: format!("construct::skills::{}", ts.name.replace('-', "_")),
        })
    }

    fn backward(&self, rs: &RustSkill) -> Result<TsSkill, Self::Error> {
        Ok(TsSkill {
            name: rs.name.clone(),
            inputs: rs.params.clone(),
            outputs: rs.returns.clone(),
            source: format!("equipment/skills/{}.ts", rs.name),
        })
    }
}

// ── BridgeRegistry ─────────────────────────────────────────────────────────

/// A registry that stores named bridges for runtime lookup.
pub struct BridgeRegistry {
    bridges: HashMap<String, String>, // name -> bridge type description
}

impl BridgeRegistry {
    pub fn new() -> Self {
        Self { bridges: HashMap::new() }
    }

    pub fn register(&mut self, name: &str, bridge_type: &str) {
        self.bridges.insert(name.to_string(), bridge_type.to_string());
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        self.bridges.get(name).map(|s| s.as_str())
    }

    pub fn list(&self) -> Vec<(&str, &str)> {
        self.bridges.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect()
    }

    pub fn remove(&mut self, name: &str) -> bool {
        self.bridges.remove(name).is_some()
    }
}

impl Default for BridgeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_value_roundtrip() {
        for v in &[TernaryValue::Negative, TernaryValue::Zero, TernaryValue::Positive] {
            assert_eq!(TernaryValue::from_i8(v.to_i8()).unwrap(), *v);
        }
    }

    #[test]
    fn test_ternary_invalid_i8() {
        assert!(TernaryValue::from_i8(2).is_none());
        assert!(TernaryValue::from_i8(-2).is_none());
    }

    #[test]
    fn test_protocol_bridge_forward() {
        let bridge = ProtocolBridge::new();
        let proto = ProtocolMessage::new("node-a", "node-b", vec![
            TernaryValue::Positive, TernaryValue::Zero, TernaryValue::Negative,
        ]);
        let oracle = bridge.forward(&proto).unwrap();
        assert_eq!(oracle.from_node, "node-a");
        assert_eq!(oracle.data, vec![1, 0, -1]);
        assert_eq!(oracle.msg_id, 1);
    }

    #[test]
    fn test_protocol_bridge_backward() {
        let bridge = ProtocolBridge::new();
        let oracle = OracleMessage::new("x", "y", vec![1, 0, -1]);
        let proto = bridge.backward(&oracle).unwrap();
        assert_eq!(proto.payload, vec![
            TernaryValue::Positive, TernaryValue::Zero, TernaryValue::Negative,
        ]);
        assert_eq!(proto.sequence, 0);
    }

    #[test]
    fn test_protocol_bridge_roundtrip() {
        let bridge = ProtocolBridge::new();
        let proto = ProtocolMessage::new("a", "b", vec![
            TernaryValue::Negative, TernaryValue::Zero, TernaryValue::Positive,
        ]);
        let oracle = bridge.forward(&proto).unwrap();
        let back = bridge.backward(&oracle).unwrap();
        assert_eq!(back.payload, proto.payload);
        assert_eq!(back.source, proto.source);
        assert_eq!(back.destination, proto.destination);
    }

    #[test]
    fn test_protocol_bridge_invalid_backward() {
        let bridge = ProtocolBridge::new();
        let oracle = OracleMessage::new("a", "b", vec![5, 0]);
        assert!(bridge.backward(&oracle).is_err());
    }

    #[test]
    fn test_codec_byte_per_trit() {
        let values = vec![TernaryValue::Negative, TernaryValue::Zero, TernaryValue::Positive];
        let encoded = CodecBridge::encode(&values, WireFormat::BytePerTrit).unwrap();
        let decoded = CodecBridge::decode(&encoded, WireFormat::BytePerTrit).unwrap();
        assert_eq!(decoded, values);
    }

    #[test]
    fn test_codec_packed() {
        let values = vec![TernaryValue::Negative, TernaryValue::Positive, TernaryValue::Zero, TernaryValue::Zero];
        let encoded = CodecBridge::encode(&values, WireFormat::Packed).unwrap();
        assert_eq!(encoded.len(), 2); // 4 trits = 2 bytes packed
        let decoded = CodecBridge::decode(&encoded, WireFormat::Packed).unwrap();
        assert_eq!(decoded, values);
    }

    #[test]
    fn test_codec_packed_odd_fails() {
        let values = vec![TernaryValue::Positive, TernaryValue::Zero, TernaryValue::Negative];
        assert!(CodecBridge::encode(&values, WireFormat::Packed).is_err());
    }

    #[test]
    fn test_codec_text() {
        let values = vec![TernaryValue::Negative, TernaryValue::Zero, TernaryValue::Positive];
        let encoded = CodecBridge::encode(&values, WireFormat::Text).unwrap();
        assert_eq!(&String::from_utf8_lossy(&encoded), "T01");
        let decoded = CodecBridge::decode(&encoded, WireFormat::Text).unwrap();
        assert_eq!(decoded, values);
    }

    #[test]
    fn test_codec_text_invalid_char() {
        assert!(CodecBridge::decode(b"X", WireFormat::Text).is_err());
    }

    #[test]
    fn test_memory_bridge_forward() {
        let bridge = MemoryBridge;
        let stm = StmEntry {
            key: "temp-reading".into(),
            value: vec![TernaryValue::Positive],
            created_at: 100,
        };
        let ltm = bridge.forward(&stm).unwrap();
        assert_eq!(ltm.key, "temp-reading");
        assert_eq!(ltm.access_count, 0);
        assert_eq!(ltm.weight, 1);
    }

    #[test]
    fn test_memory_bridge_backward() {
        let bridge = MemoryBridge;
        let ltm = LtmEntry {
            key: "pattern".into(),
            value: vec![TernaryValue::Negative],
            access_count: 42,
            weight: 10,
        };
        let stm = bridge.backward(&ltm).unwrap();
        assert_eq!(stm.key, "pattern");
        assert_eq!(stm.created_at, 0); // STM doesn't preserve LTM metadata
    }

    #[test]
    fn test_memory_bridge_roundtrip() {
        let bridge = MemoryBridge;
        let stm = StmEntry {
            key: "k".into(),
            value: vec![TernaryValue::Zero],
            created_at: 50,
        };
        let ltm = bridge.forward(&stm).unwrap();
        let back = bridge.backward(&ltm).unwrap();
        assert_eq!(back.key, stm.key);
        assert_eq!(back.value, stm.value);
    }

    #[test]
    fn test_skill_bridge_forward() {
        let bridge = SkillBridge;
        let ts = TsSkill {
            name: "detect-anomaly".into(),
            inputs: vec!["sensor-data".into()],
            outputs: vec!["is-anomaly".into()],
            source: "equipment/skills/detect-anomaly.ts".into(),
        };
        let rs = bridge.forward(&ts).unwrap();
        assert_eq!(rs.name, "detect-anomaly");
        assert_eq!(rs.module_path, "construct::skills::detect_anomaly");
        assert_eq!(rs.params, vec!["sensor-data"]);
    }

    #[test]
    fn test_skill_bridge_backward() {
        let bridge = SkillBridge;
        let rs = RustSkill {
            name: "classify".into(),
            params: vec!["input".into()],
            returns: vec!["label".into()],
            module_path: "construct::skills::classify".into(),
        };
        let ts = bridge.backward(&rs).unwrap();
        assert_eq!(ts.name, "classify");
        assert_eq!(ts.source, "equipment/skills/classify.ts");
    }

    #[test]
    fn test_skill_bridge_roundtrip() {
        let bridge = SkillBridge;
        let ts = TsSkill {
            name: "transform".into(),
            inputs: vec!["data".into()],
            outputs: vec!["result".into()],
            source: "".into(),
        };
        let rs = bridge.forward(&ts).unwrap();
        let back = bridge.backward(&rs).unwrap();
        assert_eq!(back.name, ts.name);
        assert_eq!(back.inputs, ts.inputs);
        assert_eq!(back.outputs, ts.outputs);
    }

    #[test]
    fn test_bridge_registry_register_and_get() {
        let mut reg = BridgeRegistry::new();
        reg.register("proto", "ProtocolBridge");
        assert_eq!(reg.get("proto"), Some("ProtocolBridge"));
        assert_eq!(reg.get("missing"), None);
    }

    #[test]
    fn test_bridge_registry_remove() {
        let mut reg = BridgeRegistry::new();
        reg.register("codec", "CodecBridge");
        assert!(reg.remove("codec"));
        assert!(!reg.remove("codec"));
    }

    #[test]
    fn test_bridge_registry_list() {
        let mut reg = BridgeRegistry::new();
        reg.register("a", "TypeA");
        reg.register("b", "TypeB");
        let list = reg.list();
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_codec_default_impl() {
        let bridge = CodecBridge;
        let values = vec![TernaryValue::Positive, TernaryValue::Zero];
        let encoded = bridge.forward(&values).unwrap();
        let decoded = bridge.backward(&encoded).unwrap();
        assert_eq!(decoded, values);
    }

    #[test]
    fn test_bridge_error_display() {
        let err = BridgeError::new("test error");
        assert_eq!(format!("{}", err), "BridgeError: test error");
    }
}
