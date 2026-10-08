# T70g — Doğa, portal ve kızıltaş blok animasyonları

**Amaç:** T70e döngüsüne kalan blokların `animateTick`'i: yaprak (düşen yaprak, damla), spor çiçeği,
miselyum, göz çiçeği, ateş böceği çalısı, solgun gül, asılı yosun, kuru ot sesleri, kum/düşen blok
tozu, arı kovanı balı, creaking kalbi, sculk sensörü; nether ve end portalı, end geçidi, ender sandığı,
uç çubuğu (`EndRodParticle`), kabarcık sütunu, ıslak sünger, kızıltaş tozu/tekrarlayıcı/şalter/cevher,
sarkıt ucu damlası, ağlayan obsidyen, yeniden doğma çapası, kuru ghast, potent sulfur, fırçalanabilir
blok, paratoner; yaratıcıda işaretçi (`BLOCK_MARKER`) ve biyomun ortam parçacıkları
(`AMBIENT_PARTICLES`).
**Bağımlılık:** T70f. **Boy:** L.

## Oku
Java: ilgili blok sınıflarının `animateTick` yöntemleri (`grep -ln "public void animateTick"
world/level/block/`), `world/attribute/EnvironmentAttributes.java`. Rust:
`crates/voxel-world/src/particles/animate_blocks.rs`.

## Oracle
`movement/particle-engine` `animate_tick` satırları (blok düzlemleri).

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-block-animate-parity-conformance.md`.

## Devir notu
Yapılan: `particles/animate_misc.rs` (nether portalı, spor çiçeği, miselyum, ağlayan obsidyen, yeniden
doğma çapası, uç çubuğu) ve `EndRodParticle`. Oracle: `animate-misc` ve `end-rod` bit eşit.
Bölündü: yapraklar ve kalan bloklar → T70h.
