# T70f — Kalan blokların `animateTick`'i

**Amaç:** T70e döngüsüne öteki blok sınıflarını eklemek: ateş ve ruh ateşi (duman, ses), kamp
ateşi, mumlar ve mumlu pasta, fırın/izabe fırını/tütsülük (yanarken), yaprak (düşen yaprak ve damla),
spor çiçeği, uç çubuğu, simya standı, büyü masası (kitaplık harfleri), nether ve end portalı,
miselyum, ağlayan obsidyen, sarkıt damlası, end geçidi, ışıldayan likenler ve sculk; yaratıcıda
işaretçi parçacığı (`BLOCK_MARKER`) ve biyomun ortam parçacıkları (`AMBIENT_PARTICLES`).
**Bağımlılık:** T70e, T73b. **Boy:** L.

## Oku
Java: ilgili blok sınıflarının `animateTick` yöntemleri (`grep -ln "animateTick" world/level/block/`),
`world/attribute/EnvironmentAttributes.java` (`AMBIENT_PARTICLES`). Rust:
`crates/voxel-world/src/particles/animate.rs`.

## Oracle
`movement/particle-engine` `animate_tick` satırları (blok düzlemleri).

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-block-animate-parity-conformance.md`.

## Devir notu
