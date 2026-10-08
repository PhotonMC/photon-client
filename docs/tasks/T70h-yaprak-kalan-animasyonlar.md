# T70h — Yaprak ve kalan blok animasyonları

**Amaç:** T70g'den kalan `animateTick`'ler: yapraklar (yağmurda damla, `FallingLeavesParticle` ile
düşen yaprak: renkli/renksiz, kiraz yaprağı, solgun meşe; `leafParticleChance`), göz çiçeği, ateş böceği
çalısı, solgun gül, asılı yosun, kuru ot ve ölü çalı sesleri, arı kovanı balı, creaking kalbi, sculk
sensörü, end portalı ve geçidi, ender sandığı, kabarcık sütunu, ıslak sünger, kızıltaş tozu,
tekrarlayıcı, şalter, kızıltaş cevheri, sarkıt ucu damlası, kuru ghast, potent sulfur, fırçalanabilir
blok, paratoner; yaratıcıda işaretçi (`BLOCK_MARKER`), biyomun ortam parçacıkları (`AMBIENT_PARTICLES`).
**Bağımlılık:** T70g. **Boy:** L.

## Oku
Java: ilgili blok sınıflarının `animateTick`'i, `client/particle/FallingLeavesParticle.java`,
`world/level/block/{LeavesBlock,TintedParticleLeavesBlock,UntintedParticleLeavesBlock}.java`.
Rust: `crates/voxel-world/src/particles/animate_misc.rs`.

## Oracle
`movement/particle-engine` `animate_tick` satırları.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-block-animate-parity-conformance.md`.

## Devir notu
