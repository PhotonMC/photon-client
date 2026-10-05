# T39f — Olta şamandırası

**Amaç:** `FishingHookRenderer`: şamandıra sprite'ı ve oltacıya bağlayan çizgi (`getHookedIn`, eldeki olta kolu).
**Bağımlılık:** T39. **Boy:** M.

## Oku
- Java: `client/renderer/entity/FishingHookRenderer.java`, `world/entity/projectile/FishingHook.java`.
- Rust: `crates/voxel/src/dropped_items.rs` (sprite), `crates/voxel-render/src/guardian_beam.rs` (çizgi geometrisi kalıbı).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: sprite (`fishing_hook_quad`), çizgi (`voxel-render::fishing`, oracle'da gerçek `stringVertex`), el konumu, oyunda `fishing_hooks.rs`. Kayıt: `docs/client-fishing-hook-parity-conformance.md`.
- Sapma: el konumu oracle'sız (Player kurulamadı); yerel ana kol sağ; şamandıranın istemci tiki yok.
