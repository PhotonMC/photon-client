# T78f — Kalan seviye olayları

**Amaç:** [T78e](T78e-son-seviye-olaylari.md)'nin kalanı: deneme spawner'ının olayları (3011–3014, 3017, 3019–3021: `TrialSpawner.add*Particles`, alev türünün çözümü) ve `trial_spawner_detection` zerresi (`LOOKAT_Y`). Sarkıt, kemik tozu, portal yolculuğu, kasa ve jukebox [T78g](T78g-son-seviye-olaylari.md)'ye devredildi.
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
Yapıldı: `level_events_last.rs` deneme spawner'ı olayları, `trial_spawner_detection(_ominous)` sağlayıcısı (`Kind::TrialDetection`, `LOOKAT_Y` çizimi: `photon-render/particles.rs`). Oracle: `particle-engine` `level-event-trial-spawner`, `particle-ticks` (havai fişek kıvılcımı, parlama, ejderha nefesi, deneme zerresi) ve `particle-quads` (LOOKAT_Y) satırları eşit.
