# ternary-bridge: Bridge pattern for connecting heterogeneous ternary systems

## Why This Exists

A ternary fleet isn't homogeneous. Different subsystems speak different protocols, serialize data in different wire formats, and use different memory models. The Oracle1 system uses I2I messaging. The Construct system uses Rust skills. Equipment uses TypeScript. Short-term memory is ephemeral; long-term memory has access tracking. Without bridges, none of these can talk to each other. This crate is the interop layer.

## Core Concepts

- **Bridge trait**: `Bridge<A, B>` with `forward(A) → B` and `backward(B) → A`. Any translation that can fail returns `Result`. This is the core abstraction — every bridge in the fleet implements it.
- **ProtocolBridge**: Translates between `ProtocolMessage` (ternary-protocol format: source, destination, ternary payload, sequence number) and `OracleMessage` (Oracle1 I2I format: from_node, to_node, i8 data, msg_id).
- **CodecBridge**: Encodes/decodes `Vec<TernaryValue>` to/from bytes in three wire formats: `BytePerTrit` (1 byte per trit), `Packed` (2 trits per byte), `Text` (T/0/1 characters).
- **MemoryBridge**: Promotes `StmEntry` (short-term memory) to `LtmEntry` (long-term memory with access tracking) and demotes back.
- **SkillBridge**: Converts between `TsSkill` (TypeScript/Equipment descriptor) and `RustSkill` (Rust/Construct descriptor), translating module paths and parameter naming.
- **BridgeRegistry**: Named registry for looking up bridges at runtime by name.

## Quick Start

```toml
[dependencies]
ternary-bridge = "0.1"
```

```rust
use ternary_bridge::*;

// Protocol bridge: translate between systems
let bridge = ProtocolBridge::new();
let proto = ProtocolMessage::new("sensor-a", "controller", vec![
    TernaryValue::Positive, TernaryValue::Zero, TernaryValue::Negative,
]);
let oracle = bridge.forward(&proto).unwrap();
assert_eq!(oracle.data, vec![1, 0, -1]);

// Codec: encode ternary to bytes
let values = vec![TernaryValue::Negative, TernaryValue::Positive];
let packed = CodecBridge::encode(&values, WireFormat::Packed).unwrap();
let decoded = CodecBridge::decode(&packed, WireFormat::Packed).unwrap();
assert_eq!(decoded, values);
```

## API Overview

| Type | Description |
|------|-------------|
| `TernaryValue` | Enum: `Negative`, `Zero`, `Positive` — maps to -1, 0, +1 |
| `Bridge<A, B>` | Trait with `forward` and `backward` translation methods |
| `BridgeError` | Error type for failed translations |
| `WireFormat` | Enum: `BytePerTrit`, `Packed`, `Text` |
| `ProtocolMessage` | ternary-protocol message (source, dest, ternary payload) |
| `OracleMessage` | Oracle1 I2I message (from_node, to_node, i8 data) |
| `ProtocolBridge` | Translates Protocol ↔ Oracle messages |
| `CodecBridge` | Encodes/decodes ternary to/from wire formats |
| `StmEntry` / `LtmEntry` | Short-term and long-term memory entries |
| `MemoryBridge` | Translates STM ↔ LTM entries |
| `TsSkill` / `RustSkill` | Skill descriptors in TypeScript and Rust formats |
| `SkillBridge` | Translates TS ↔ Rust skill descriptors |
| `BridgeRegistry` | Named registry for runtime bridge lookup |

## How It Works

Every bridge implements the `Bridge<A, B>` trait with two directions. Forward goes A→B, backward goes B→A. Both can fail with `BridgeError`. The `ProtocolBridge` assigns auto-incrementing sequence numbers on forward translation. The `CodecBridge` uses static methods since encoding is stateless. Packed format maps ternary values to 2-bit nibbles (Neg=3, Zero=0, Pos=1) and packs two per byte. The `MemoryBridge` discards LTM metadata (access_count, weight) when demoting to STM, since STM doesn't track those.

## Known Limitations

- Packed wire format requires an even number of trits. Odd-length payloads are rejected.
- Text wire format uses single ASCII chars: 'T' for negative, '0' for zero, '1' for positive. Non-ASCII or other characters cause decode errors.
- MemoryBridge is lossy backward: LTM access_count and weight are lost when converting to STM.
- SkillBridge assumes a flat module namespace (`construct::skills::{name}`). It doesn't handle nested module hierarchies.
- BridgeRegistry stores type descriptions as strings, not typed bridge instances. It's a catalog, not a dependency injection container.

## Use Cases

1. **Fleet interoperability**: A sensor node sends ternary-protocol messages that need to be understood by an Oracle1-based controller. ProtocolBridge translates in both directions.
2. **Wire format migration**: Legacy systems use byte-per-trit encoding; new systems use packed format. CodecBridge handles transcoding.
3. **Skill migration from TypeScript to Rust**: An Equipment skill written in TypeScript needs a Construct equivalent. SkillBridge generates the Rust descriptor with the correct module path.
4. **Memory promotion**: When a short-term memory pattern proves reliable, MemoryBridge promotes it to long-term storage with access tracking.

## Ecosystem Context

Central to the SuperInstance interop story. Depends on nothing external. Used by `ternary-protocol` (message bridging), `ternary-memory` (STM↔LTM), `ternary-ensign` (skill bridging), and `ternary-constellation` (bundling bridges into deployable units). If you need two ternary subsystems to talk, you need this crate.

## License

MIT

## See Also
- **ternary-protocol** — related
- **ternary-channel** — related
- **ternary-mesh** — related
- **ternary-flux** — related
- **ternary-constellation** — related

