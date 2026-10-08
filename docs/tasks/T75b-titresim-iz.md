# T75b — Titreşim ve iz parçacıkları

**Amaç:** `VibrationSignalParticle` (`VibrationParticleOption`: hedef konum kaynağı — blok ya da
entity — ve varış süresi; yönelen dörtgen) ve `TrailParticle` (`TrailParticleOption`: hedef, renk,
süre). İkisi de ağdan konum kaynağı/hedef okur; `ParticleData`'ya eklenir.
**Bağımlılık:** T75. **Boy:** M.

## Oku
Java: `client/particle/{VibrationSignalParticle,TrailParticle}.java`,
`core/particles/{VibrationParticleOption,TrailParticleOption}.java`,
`world/level/gameevent/PositionSource*.java`. Rust: `crates/photon-world/src/particles/attract.rs`.

## Oracle
`particle-ticks` satırları (gerçek sağlayıcılar, seçenekler satırdan).

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-particle-attract-parity-conformance.md`.

## Devir notu
