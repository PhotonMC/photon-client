# T64b — Kalan oyun sesleri ve müzik

**Amaç:** T64a'nın hareket seslerini (adım, yüzme) gerçek Java'ya (`MovementOracle`, sessiz `SoundManager` kaydı) karşı karşılaştırmak; eksik hareket seslerini eklemek: iniş/düşme (`causeFallDamage`, blok `fallOn`), su sıçraması, kabarcık sütununa girme. Hasar [T64c](T64c-ses-tetikleri-son.md)'e, ortam, `LevelEvent` ve müzik [T64d](T64d-ses-tetikleri-ortam-muzik.md)'ye ayrıldı.
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
- Yapıldı: gerçek `LocalPlayer` sahnesinin çaldığı sesler `local-player-ticks` satırlarına `sounds` olarak girdi (olay, kaynak, ses, perde, yer; bit bit). Rust: iniş (`fall_on`: saman, bal, toz karı, yatak, sivri damlataş; `cause_fall_damage`, blok düşme sesi), su sıçraması, `BubbleColumnAmbientSoundHandler`, varlığın kendi rastgelesi (`Entity.random`).
- Oracle: rastgele sesler karşılaştırılmaz (ortam, bal kayması; yüzme/sıçrama perdesi `0`); binekteki oyuncunun sesleri alınmaz (düşüş yolcuya aktarımı yok). Kayıt: `docs/client-block-sounds-conformance.md`.
- Açık: [T64d](T64d-ses-tetikleri-ortam-muzik.md).
