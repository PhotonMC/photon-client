# T74b — Düşen toz ve havai fişek parçacıkları

**Amaç:** `FallingDustParticle` (bloğun toz rengi: `FallingBlock.getDustColor`, renk kaynağı ya da
harita rengi; dönen düşüş) ve `FireworkParticles` (kıvılcım, parlama, başlatıcının patlama biçimleri,
renk solması, iz ve parıltı; `ClientLevel.createFireworks`).
**Bağımlılık:** T74. **Boy:** M.

## Oku
Java: `client/particle/{FallingDustParticle,FireworkParticles}.java`, `world/level/block/FallingBlock.java`
ve alt sınıflarının `getDustColor`'ı, `MapColor`. Rust: `crates/voxel-world/src/particles/colour.rs`.

## Oracle
`particle-ticks`/`particle-engine` satırları, gerçek sağlayıcılarla.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-particle-colour-parity-conformance.md`.

## Devir notu
Yapılan: `particles/falling_dust.rs`, `animate_falling.rs` (düşen blok tozu, kum çöl sesi); harita rengi
ve toz rengi tabloları oracle'dan (`tools/generate/map_colors.py`), her koşuda karşılaştırılır.
Görünmez blokta tohum harcanmıyor. Oracle: `animate-falling`, `falling-dust-add`, `block-map-colors` eşit.
Bölündü: havai fişek → T74c.
