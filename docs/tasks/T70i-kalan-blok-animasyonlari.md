# T70i — Kalan blok animasyonları

**Amaç:** Kalan `animateTick`'ler: yaprakların yağmurdaki damlası (`Level.isRainingAt`: yağış biyomu,
gökyüzü, `MOTION_BLOCKING`), göz çiçeği, ateş böceği çalısı, solgun gül, asılı yosun, kuru ot ve ölü
çalı sesleri, arı kovanı balı, creaking kalbi, sculk sensörü, end portalı ve geçidi, ender sandığı,
kabarcık sütunu, ıslak sünger, kızıltaş tozu, tekrarlayıcı, şalter, kızıltaş cevheri, sarkıt ucu
damlası, kuru ghast, potent sulfur, fırçalanabilir blok, paratoner; yaratıcıda işaretçi
(`BLOCK_MARKER`), biyomun ortam parçacıkları (`AMBIENT_PARTICLES`).
**Bağımlılık:** T70h. **Boy:** L.

## Oku
Java: ilgili blok sınıflarının `animateTick`'i, `world/level/block/LeavesBlock.java`.
Rust: `crates/photon-world/src/particles/animate_misc.rs`.

## Oracle
`movement/particle-engine` `animate_tick` satırları.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-block-animate-parity-conformance.md`.

## Devir notu
Yapıldı: `particles/animate_redstone.rs`: şalter, kızıltaş cevheri, tekrarlayıcı, kızıltaş tozu, ıslak sünger, kabarcık sütunu, ender sandığı, sculk sensörleri (`animate-redstone` oracle satırı eşit). Kalan: T70j.
