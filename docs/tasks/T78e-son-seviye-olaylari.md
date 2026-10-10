# T78e — Son seviye olayları

**Amaç:** `LevelEventHandler`'ın kalanı: 1504 sarkıt damlası (`PointedDripstoneBlock.spawnDripParticle`),
1505 kemik tozu (`BoneMealItem.addGrowthParticles`), 2006 ejderha nefesi (`DragonBreathParticle`,
`PowerParticleOption`), 3000–3021 (geçit, sculk, titreşim, deneme spawner'ı, kasa, ...), küresel
olaylar 1023/1028/1038 (kameradan iki blok ötede), 1032 portal yolculuğu, jukebox 1010/1011.
**Bağımlılık:** T78b. **Boy:** L.

## Oku
Java: `client/renderer/LevelEventHandler.java` (440–700), `world/item/BoneMealItem.java`,
`world/level/block/PointedDripstoneBlock.java`, `client/particle/DragonBreathParticle.java`.
Rust: `crates/photon-world/src/particles/level_events_more.rs`.

## Oracle
`movement/particle-engine` `level_event` satırları.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-level-event-parity-conformance.md`.

## Devir notu
Yapıldı: `particles/level_events_last.rs`: 2006 (+`dragon_breath` sağlayıcısı ve `PowerParticleOption` paket okuması), 3000–3007, 3009, 3018, küresel olaylar 1023/1028/1038 (`LevelEvent::Global`, paket yönlendirmesi). Oracle `particle-engine` 8 satır eşit. Kalan: T78f.
Sapma: mesafe gecikmeli ses yok (oracle erteli sesi hemen sayar).
