# T39b — Trident

**Amaç:** `ThrownTridentRenderer` (model, parıltı katmanı, ısı: `loyalty` dönüşünde dönen yön), `ThrownTrident` istemci durumu (`ID_FOIL`, `ID_LOYALTY`, `dealtDamage`), `AbstractArrow` ortak kodu (T39).
**Bağımlılık:** T39. **Boy:** M.

## Oku
- Java: `client/renderer/entity/ThrownTridentRenderer.java`, `client/model/object/projectile/TridentModel.java`, `world/entity/projectile/arrow/ThrownTrident.java`.
- Rust: `crates/photon/src/objects/arrows.rs`, `crates/photon-world/src/objects/arrow.rs`, `crates/photon-render/src/objects/projectile.rs`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `trident_pose` ve `setup_trident`, `arrows.rs`'e trident dalı (arka yüz ayıklama yok). Kayıt: `docs/records/client-trident-parity-conformance.md`.
- Sapma: büyü parıltısı yok (T41e parıltı hattı); `ID_FOIL`/`ID_LOYALTY` okunmuyor.
