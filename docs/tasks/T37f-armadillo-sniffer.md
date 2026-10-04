# T37f — Armadillo ve sniffer

**Amaç:** `AdultArmadilloModel`/`BabyArmadilloModel` (top olma ve açılma durumları), `SnifferModel`/`SniffletModel` (koklama, kazma, yürüme animasyonları), ilgili renderer ve istemci durumları.
**Bağımlılık:** T37, T43b. **Boy:** L.

## Oku
- Java: `world/entity/animal/armadillo/Armadillo.java`, `world/entity/animal/sniffer/Sniffer.java`, `client/model/animal/{armadillo,sniffer}/*.java`, `client/renderer/entity/{ArmadilloRenderer,SnifferRenderer}.java`.
- Rust: `crates/voxel-render/src/mob_model.rs`, `crates/voxel-world/src/animation_state.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
