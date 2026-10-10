# T78f — Kalan seviye olayları

**Amaç:** [T78e](T78e-son-seviye-olaylari.md)'nin kalanı: 1504 sarkıt damlası (`PointedDripstoneBlock.spawnDripParticle`), 1505 kemik tozu (`BoneMealItem.addGrowthParticles`), 1032 portal yolculuğu (yerel ortam sesi), deneme spawner'ı (3011–3014, 3017, 3019–3021: `TrialSpawner.add*Particles`, alev türünün çözümü) ve kasa (3015, 3016: `VaultBlockEntity.Client`; blok entity'si ister), jukebox 1010/1011 (`playJukeboxSong`, çevredeki varlıklara bildirim).
**Bağımlılık:** T78e. **Boy:** L.

## Oku
Java: `client/renderer/LevelEventHandler.java` (255–300, 440–700), `world/level/block/PointedDripstoneBlock.java`, `world/item/BoneMealItem.java`, `world/level/block/entity/trialspawner/TrialSpawner.java`, `world/level/block/entity/vault/VaultBlockEntity.java`. Rust: `crates/photon-world/src/particles/level_events_last.rs` (örnek).

## Oracle
`movement/particle-engine` `level_event` satırları.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-level-event-parity-conformance.md`.

## Devir notu
(boş)
