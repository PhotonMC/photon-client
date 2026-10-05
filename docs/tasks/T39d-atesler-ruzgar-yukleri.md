# T39d — Ateş topları ve rüzgâr yükleri

**Amaç:** `fireball`, `small_fireball`, `dragon_fireball` (ve `breeze_wind_charge`, `wind_charge`): `ItemSupplier` sprite'ı, `DragonFireballRenderer`, `WindChargeRenderer` (kayan doku).
**Bağımlılık:** T39. **Boy:** M.

## Oku
- Java: `client/renderer/entity/{DragonFireballRenderer,WindChargeRenderer}.java`, `client/renderer/entity/ThrownItemRenderer.java` (ateş topları da bunu kullanır), `client/model/object/projectile/WindChargeModel.java`.
- Rust: `crates/voxel-render/src/mob_model.rs`, `crates/voxel/src/breeze` kalıbı (kayan doku: `mob_vertices_scrolled`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
