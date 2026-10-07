# T39d — Ejderha topu ve rüzgâr yükleri

**Amaç:** `dragon_fireball` ve `breeze_wind_charge`, `wind_charge` (`fireball` ve `small_fireball` T39c'de çözüldü): `ItemSupplier` sprite'ı, `DragonFireballRenderer`, `WindChargeRenderer` (kayan doku).
**Bağımlılık:** T39. **Boy:** M.

## Oku
- Java: `client/renderer/entity/{DragonFireballRenderer,WindChargeRenderer}.java`, `client/renderer/entity/ThrownItemRenderer.java` (ateş topları da bunu kullanır), `client/model/object/projectile/WindChargeModel.java`.
- Rust: `crates/voxel-render/src/entity/mob_model.rs`, `crates/voxel/src/breeze` kalıbı (kayan doku: `mob_vertices_scrolled`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `setup_wind_charge`, `mob_vertices_scrolled_posed`, `dragon_fireball_quad`; oyunda `charges.rs`. Kayıt: `docs/records/client-charges-parity-conformance.md`.
- Sapma: kayma değeri oracle'da yok; dış çizgi yok; rüzgâr yükü canlıda görülemedi (sunucu hemen siliyor).
