# T78g — Son seviye olayları

**Amaç:** [T78f](T78f-kalan-seviye-olaylari.md)'nin kalanı: 1504 sarkıt damlası (`PointedDripstoneBlock.spawnDripParticle`; `WATER_EVAPORATES` ve `DEFAULT_DRIPSTONE_PARTICLE` çevre öznitelikleri ister, sarkıtın `animateTick`'iyle ortak), 1505 kemik tozu (`BoneMealItem.addGrowthParticles`: her `BonemealableBlock`'un türü ve `getParticlePos`'u), 1032 portal yolculuğu (yerel ortam sesi), kasa 3015/3016 (`VaultBlockEntity.Client`; blok entity'si ister), jukebox 1010/1011 (`playJukeboxSong`, çevredeki varlıklara bildirim).
**Bağımlılık:** T78f. **Boy:** L.

## Oku
Java: `client/renderer/LevelEventHandler.java` (255–300, 440–700), `world/level/block/PointedDripstoneBlock.java`, `world/item/BoneMealItem.java`, `world/level/block/entity/vault/VaultBlockEntity.java`. Rust: `crates/photon-world/src/particles/level_events_last.rs`.

## Oracle
`movement/particle-engine` `level_event` satırları.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-level-event-parity-conformance.md`.

## Devir notu
(boş)
