# T73b — Damlama parçacıkları

**Amaç:** `DripParticle` ailesi: su, lav, bal, obsidyen gözyaşı, dripstone damlaları, nektar,
spor çiçeği, ...; asılı (`DripHangParticle`), düşen (`FallAndLandParticle`), inen (`DripLandParticle`)
hâlleri ve birinden ötekine geçiş (`ParticleOptions` ile yeni parçacık isteme).
**Bağımlılık:** T73. **Boy:** M.

## Oku
Java: `client/particle/DripParticle.java` ve iç sınıfları, `ParticleResources` kayıtları.
Rust: `crates/voxel-world/src/particles/water.rs` kalıbı, motorun tik içi ekleme yolu (T72 lav).

## Oracle
`particle-ticks` ve motor satırları (`particle-engine`), gerçek sağlayıcılarla.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-particle-water-parity-conformance.md` ya da yeni kayıt.

## Devir notu
