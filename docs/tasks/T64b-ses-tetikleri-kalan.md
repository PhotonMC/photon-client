# T64b — Kalan oyun sesleri ve müzik

**Amaç:** T64a'dan kalanlar: atlama/iniş/düşme sesleri, oyuncu hasarı ve ölüm, ametist tınısı, su sıçraması, yağmur ve ortam (mağara, su altı) döngüleri, kapı/sandık gibi `LevelEvent` sesleri, eşya kullanım sesleri, `MusicManager` (menü/oyun müziği, bekleme, tohumlu seçim); T64a'nın tetiklerinin gerçek Java'ya (MovementOracle, `Level.playSound`) karşı karşılaştırılması.
**Bağımlılık:** T64. **Boy:** L.

## Oku
- Java: `client/sounds/MusicManager.java`, `client/renderer/LevelEventHandler.java`, `world/entity/{Entity,LivingEntity}.java` (`playSound`, `getFallSounds`, `playHurtSound`), `client/sounds/AmbientSoundHandler`/`UnderwaterAmbientSoundHandler`, `BiomeAmbientSoundsHandler`.
- Rust: `crates/voxel-world/src/local_player/sounds.rs`, `crates/voxel/src/sound.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Mob sesleri (aile görevlerinde), altyazılar.

## Belgeler
`docs/client-block-sounds-conformance.md` sapma listesi.

## Devir notu
(boş)
