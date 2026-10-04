# T37e — Creaking

**Amaç:** `CreakingModel` (yürüyüş ve saldırı `KeyframeAnimation`'ı, bükülme), `CreakingRenderer` (emissive göz katmanı), `Creaking` istemci durumu (`IS_ACTIVE`, `CAN_MOVE`, `IS_TEARING_DOWN`, ölümde çökme).
**Bağımlılık:** T37, T43b. **Boy:** M.

## Oku
- Java: `world/entity/monster/creaking/Creaking.java`, `client/model/monster/creaking/CreakingModel.java`, `client/renderer/entity/{CreakingRenderer,state/CreakingRenderState}.java`.
- Rust: `crates/voxel-render/src/mob_model.rs`, `crates/voxel-world/src/animation_state.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
