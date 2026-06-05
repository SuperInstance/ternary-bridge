# ternary-bridge

**Bridge pattern for connecting heterogeneous ternary systems**

[![ternary](https://img.shields.io/badge/ecosystem-ternary-blue)](https://github.com/orgs/SuperInstance/repositories?q=ternary)
[![tests](https://img.shields.io/badge/tests-22-green)]()

## Overview

Bridge pattern for connecting heterogeneous ternary systems.

In a tri-axial ternary fleet, different subsystems speak different protocols,
use different wire formats, and store data in different memory models. This
crate provides the `Bridge` trait and concrete bridge implementations:
`ProtocolBridge`, `CodecBridge`, `MemoryBridge`, and `SkillBridge` — the
interop layer that makes the fleet talk to each other.

## Architecture

- **`BridgeError`** — core data structure
- **`ProtocolMessage`** — core data structure
- **`OracleMessage`** — core data structure
- **`ProtocolBridge`** — core data structure
- **`CodecBridge`** — core data structure
- **`StmEntry`** — core data structure
- **`LtmEntry`** — core data structure
- **`MemoryBridge`** — core data structure
- **`TsSkill`** — core data structure
- **`RustSkill`** — core data structure
- **`SkillBridge`** — core data structure
- **`BridgeRegistry`** — core data structure
- **`TernaryValue`** — state enumeration
- **`WireFormat`** — state enumeration

### Traits

- **`Bridge`** — shared behavior contract

### Key Functions

- `to_i8()`
- `from_i8()`
- `new()`
- `new()`
- `new()`
- `new()`
- `encode()`
- `decode()`
- `new()`
- `register()`
- ... and 3 more

## Why Ternary?

The balanced ternary system {-1, 0, +1} (also known as Z₃) is the mathematically optimal discrete encoding:
- **More expressive than binary**: three states capture positive, neutral, and negative
- **Natural for decisions**: accept/reject/abstain, buy/hold/sell, agree/disagree/neutral
- **Self-balancing**: the 0 state acts as a universal screen, preventing pathological lock-in
- **Z₃ cyclic dynamics**: rock-paper-scissors is the only natural coordination mechanism

## Stats

| Metric | Value |
|--------|-------|
| Lines of Rust | 633 |
| Test count | 22 |
| Public types | 14 |
| Public functions | 13 |

## Ecosystem

This crate is part of the **[SuperInstance Ternary Fleet](https://github.com/orgs/SuperInstance/repositories?q=ternary)**:

- **[ternary-core](https://github.com/SuperInstance/ternary-core)** — shared traits and Z₃ arithmetic
- **[ternary-grid](https://github.com/SuperInstance/ternary-grid)** — spatial grid with {-1, 0, +1} cells
- **[ternary-graph](https://github.com/SuperInstance/ternary-graph)** — ternary-weighted graph algorithms
- **[ternary-automata](https://github.com/SuperInstance/ternary-automata)** — three-state cellular automata
- **[ternary-compiler](https://github.com/SuperInstance/ternary-compiler)** — expression compiler and optimizer

200+ crates. 4,300+ tests. One pattern.

## Research Context

The ternary approach connects to several active research areas:
- **Ternary Neural Networks** (TNNs): weights constrained to {-1, 0, +1} for efficient inference
- **Huawei's ternary chip**: 7nm ternary silicon with 60% less power consumption
- **Active inference**: free energy minimization naturally maps to ternary action selection
- **Cyclic dominance**: RPS dynamics maintain biodiversity in spatial ecology
- **Z₃ group theory**: the only algebraic group on three elements is cyclic addition mod 3

## Usage

```toml
[dependencies]
ternary-bridge = "0.1.0"
```

```rust
use ternary_bridge;
```

## License

MIT
