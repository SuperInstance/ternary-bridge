# ternary-bridge

**Bridge pattern for connecting heterogeneous ternary systems with protocol and codec translation.**

`ternary-bridge` provides the interop layer that makes different ternary subsystems communicate. It defines a generic `Bridge<A, B>` trait with `forward()` and `backward()` translations, plus concrete implementations for protocol bridging, wire-format codec conversion, memory model translation, and skill descriptor mapping.

## Why It Matters

In a fleet of ternary systems, different subsystems inevitably speak different dialects: one uses byte-per-trit wire encoding, another uses packed nibbles; one speaks the internal ternary-protocol format, another uses an external I2I oracle format. Without a structured bridge layer, translation logic scatters across the codebase as ad-hoc converters that are untestable, unmaintainable, and unverifiable.

The bridge pattern solves this by defining a **bidirectional translation contract**:

$$\text{forward}: A \to B, \qquad \text{backward}: B \to A$$

Every bridge implementation must provide both directions, enabling roundtrip verification: $\text{backward}(\text{forward}(x)) \approx x$. This crate provides four production bridges and a registry for runtime lookup.

## How It Works

### The Bridge Trait

The core abstraction is a generic trait parameterized by source type `A`, target type `B`, and an error type:

```rust
pub trait Bridge<A, B> {
    type Error;
    fn forward(&self, value: &A) -> Result<B, Self::Error>;
    fn backward(&self, value: &B) -> Result<A, Self::Error>;
}
```

Translations can fail — for example, decoding a byte value outside $\{-1, 0, +1\}$ yields `BridgeError`. The `Result` return type forces callers to handle translation failures explicitly.

**Complexity:** Translation complexity is $O(n)$ in the payload size for all implemented bridges, involving element-wise mapping.

### Protocol Bridge

Translates between internal `ProtocolMessage` and external `OracleMessage` formats:

| Field | ProtocolMessage | OracleMessage |
|-------|-----------------|---------------|
| Source | `source: String` | `from_node: String` |
| Destination | `destination: String` | `to_node: String` |
| Payload | `Vec<TernaryValue>` | `Vec<i8>` |
| Sequence | `sequence: u32` | `msg_id: u32` |

Forward: map each `TernaryValue` to its `i8` representation ($-1, 0, +1$).
Backward: parse each `i8` back to `TernaryValue`, failing on out-of-range values.

### Codec Bridge

Encodes/decodes ternary data in three wire formats:

| Format | Encoding | Density | Use Case |
|--------|----------|---------|----------|
| `BytePerTrit` | 1 trit → 1 byte (0xFF, 0x00, 0x01) | 1 trit/byte | Debugging, alignment-safe |
| `Packed` | 2 trits → 1 byte (nibble pairs) | 2 trits/byte | Bandwidth-constrained |
| `Text` | 1 trit → 1 char ('T', '0', '1') | 1 trit/byte | Human-readable |

**Packed format encoding:**

$$\text{byte} = (\text{nibble}_{\text{high}} \ll 4) \;|\; \text{nibble}_{\text{low}}$$

where the nibble mapping is: Negative → 3, Zero → 0, Positive → 1. This choice ensures that the zero trit maps to the zero nibble (alignment with binary protocols).

**Constraint:** Packed format requires an even number of trits; odd-length payloads return `BridgeError`.

**Complexity:** $O(n)$ for encoding/decoding $n$ trits across all formats.

### Memory Bridge

Translates between ephemeral short-term memory (STM) and persistent long-term memory (LTM):

| Property | StmEntry | LtmEntry |
|----------|----------|----------|
| Key | `String` | `String` |
| Value | `Vec<TernaryValue>` | `Vec<TernaryValue>` |
| Metadata | `created_at: u64` | `access_count: u32`, `weight: u32` |

Forward (STM → LTM): promote ephemeral data to persistent storage with initial access count 0 and weight 1.
Backward (LTM → STM): extract data from persistent storage, discarding access metadata.

### Skill Bridge

Translates skill descriptors between TypeScript (Equipment) and Rust (Construct) formats:

| Property | TsSkill | RustSkill |
|----------|---------|-----------|
| Name | `name: String` | `name: String` |
| Inputs | `inputs: Vec<String>` | `params: Vec<String>` |
| Outputs | `outputs: Vec<String>` | `returns: Vec<String>` |
| Source | `source: String` | `module_path: String` |

Forward generates the Rust module path: `construct::skills::{name_with_dashes_to_underscores}`.
Backward generates the TypeScript source path: `equipment/skills/{name}.ts`.

### Bridge Registry

A simple `HashMap<String, String>` registry maps bridge names to type descriptions, enabling runtime discovery of available bridges without compile-time coupling.

**Complexity:** $O(1)$ average for register/get/remove; $O(n)$ for listing all bridges.

## Quick Start

```toml
[dependencies]
ternary-bridge = "0.1"
```

```rust
use ternary_bridge::{Bridge, ProtocolBridge, CodecBridge, WireFormat, TernaryValue,
                     MemoryBridge, StmEntry, LtmEntry, SkillBridge, TsSkill, BridgeRegistry};

// Protocol bridging: ternary-protocol → Oracle I2I
let bridge = ProtocolBridge::new();
let msg = ternary_bridge::ProtocolMessage::new("node-a", "node-b", vec![
    TernaryValue::Positive, TernaryValue::Zero, TernaryValue::Negative,
]);
let oracle = bridge.forward(&msg).unwrap();
assert_eq!(oracle.data, vec![1, 0, -1]);

// Codec: encode ternary data in packed format
let values = vec![TernaryValue::Negative, TernaryValue::Positive,
                  TernaryValue::Zero, TernaryValue::Zero];
let encoded = CodecBridge::encode(&values, WireFormat::Packed).unwrap();
assert_eq!(encoded.len(), 2); // 4 trits → 2 bytes
let decoded = CodecBridge::decode(&encoded, WireFormat::Packed).unwrap();
assert_eq!(decoded, values);

// Memory promotion: STM → LTM
let stm = StmEntry {
    key: "temp".into(),
    value: vec![TernaryValue::Positive],
    created_at: 42,
};
let ltm = MemoryBridge.forward(&stm).unwrap();
assert_eq!(ltm.access_count, 0);

// Registry
let mut registry = BridgeRegistry::new();
registry.register("protocol", "ProtocolBridge<ProtocolMessage, OracleMessage>");
```

## API

| Type | Purpose |
|------|---------|
| `Bridge<A,B>` | Core bidirectional translation trait |
| `BridgeError` | Translation failure with message |
| `TernaryValue` | The $\{-1, 0, +1\}$ value type |
| `WireFormat` | BytePerTrit / Packed / Text encoding selector |
| `ProtocolMessage` / `OracleMessage` | Message types for protocol bridging |
| `ProtocolBridge` | Internal ↔ external protocol translator |
| `CodecBridge` | Wire-format encoder/decoder |
| `StmEntry` / `LtmEntry` | Short-term and long-term memory entries |
| `MemoryBridge` | STM ↔ LTM promoter |
| `TsSkill` / `RustSkill` | Skill descriptors for TS and Rust |
| `SkillBridge` | TypeScript ↔ Rust skill translator |
| `BridgeRegistry` | Named bridge lookup |

## Architecture Notes

Bridges are the **information channels** of the SuperInstance conservation law **γ + η = C**. Each bridge translates representations while preserving information content — the $\gamma$ (structured meaning) passes through unchanged, while only the encoding (carrier of $\eta$, the representational overhead) transforms.

A perfect bridge is **information-preserving**: $\text{backward}(\text{forward}(x)) = x$. Any deviation introduces $\eta$ — representational entropy from lossy translation. The packed codec format minimizes $\eta$ by using the fewest bits per trit (2 trits/byte vs. 1 trit/byte), while the text format trades higher $\eta$ for human readability.

The bridge registry maps to the conservation principle's requirement that all channels between subsystems be accounted for: unregistered bridges are information leaks that can violate $\gamma + \eta \leq C$.

## References

- Gamma, E. et al. *Design Patterns: Elements of Reusable Object-Oriented Software.* Addison-Wesley, 1994. — Bridge pattern (structural).
- Tanenbaum, A.S. & Wetherall, D.J. *Computer Networks.* 5th ed., Ch. 3, on protocol translation and encoding.
- Kleppmann, M. *Designing Data-Intensive Applications.* O'Reilly, 2017. Ch. 4, on encoding and evolution.
- Stonebraker, M. *Operating System Support for Database Management.* CACM 24(7), 1981. — Data model translation overhead.

## License

MIT
