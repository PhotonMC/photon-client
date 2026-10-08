# T73b — Damlama parçacıkları

**Amaç:** `DripParticle` ailesi: su, lav, bal, obsidyen gözyaşı, dripstone damlaları, nektar,
spor çiçeği, ...; asılı (`DripHangParticle`), düşen (`FallAndLandParticle`), inen (`DripLandParticle`)
hâlleri ve birinden ötekine geçiş (`ParticleOptions` ile yeni parçacık isteme).
**Bağımlılık:** T73. **Boy:** M.

## Oku
Java: `client/particle/DripParticle.java` ve iç sınıfları, `ParticleResources` kayıtları.
Rust: `crates/photon-world/src/particles/water.rs` kalıbı, motorun tik içi ekleme yolu (T72 lav).

## Oracle
`particle-ticks` ve motor satırları (`particle-engine`), gerçek sağlayıcılarla.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-particle-water-parity-conformance.md` ya da yeni kayıt.

## Devir notu
Yapılan: `particles/drip.rs` (17 tür, asılı/düşen/inen aşamaları, soğuma rengi, iniş sesleri);
parçacık tiki ses isteyebiliyor (`Emitted`, motor tohum çeker); `look.rs` çizim sorgularını ayırdı.
Oracle: 16 tek satır, `drip-stages` ve `animate-drips` bit eşit. Geçirimsiz etiketi düzeltildi.
Açık: sarkıt ucu damlası (T70f/T78b).
