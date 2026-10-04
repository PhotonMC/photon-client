# T37d — Warden

**Amaç:** `WardenModel` (`KeyframeAnimation`: yürüyüş, saldırı, kükreme, ses dalgası, emerge/dig durumları), `WardenRenderer` ve katmanları (kalp atışı, parlayan göğüs ve dokunaçlar, emissive katman), `Warden` istemci animasyon durumları.
**Bağımlılık:** T37, T43b. **Boy:** L.

## Oku
- Java: `world/entity/monster/warden/Warden.java`, `client/model/monster/warden/WardenModel.java`, `client/renderer/entity/{WardenRenderer,state/WardenRenderState}.java`, `layers/LivingEntityEmissiveLayer.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`animate`, `setup_*`), `crates/voxel-world/src/animation_state.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
