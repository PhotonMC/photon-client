# T64d — Ortam, olay sesleri ve müzik

**Amaç:** T64b'den kalanlar: oyuncu hasarı ve ölüm sesi (`playHurtSound`, `EntityEvent`/`DamageEvent`), ametist tınısı, yağmur, mağara ve su altı döngüleri (`UnderwaterAmbientSoundHandler`, `BiomeAmbientSoundsHandler`; rastgele oldukları için oracle'da tohum gerekir), kapı/sandık gibi `LevelEvent` sesleri, eşya kullanım sesleri, binekteki oyuncunun düşüş sesleri (`propagateFallToPassengers`), `MusicManager`.
**Bağımlılık:** T64c. **Boy:** L.

## Oku
- Java: `client/sounds/MusicManager.java`, `client/renderer/LevelEventHandler.java`, `world/entity/LivingEntity.java` (`playHurtSound`), `client/resources/sounds/{Underwater,BiomeAmbient}*Handler.java`.
- Rust: `crates/voxel-world/src/local_player/sounds.rs`, `crates/voxel/src/sound.rs`, `docs/client-block-sounds-conformance.md`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Mob sesleri (aile görevlerinde), altyazılar.

## Belgeler
`docs/client-block-sounds-conformance.md` sapma listesi.

## Devir notu
(boş)
