# T61 — `Sound`/`SoundEntity`/`StopSound` paketleri ve `SoundEngine` kanal parametreleri

**Amaç:** Sunucu ses paketlerini ayrıştır, `SoundEngine.play` yolunu (ses, perde, konum, mesafe sönümü, göreli, döngü, kategori çarpanı) ve `stop` yolunu saf mantık olarak yaz; kanalı arayüz arkasında tut (çıkış T63).
**Bağımlılık:** T60. Ses yığını kararı: [`audio-backend-decision.md`](../audio-backend-decision.md) (çıkış `cpal`, çözücü `lewton`).

## Oku
- Java: `client/sounds/{SoundEngine,ChannelAccess,SoundEngineExecutor,SoundEventListener}.java`, `network/protocol/game/{ClientboundSoundPacket,ClientboundSoundEntityPacket,ClientboundStopSoundPacket}.java`, `client/resources/sounds/SoundInstance.java`.
- Rust: `crates/voxel-net/src/connection/play.rs` (paket işleyici kalıbı), `crates/voxel-world/src/` (entity konumu).

## Yapılacaklar
1. Üç paketi ayrıştır ve oyun durumuna yaz (olay, kaynak, konum, ses, perde, tohum).
2. `SoundEngine.play`: `calculateVolume`, `calculatePitch`, mesafe ve dinleyiciye göre sönüm parametreleri, kanal havuzu sınırı (30 durağan), duraklatma kategorileri; çıktı: kanalı süren `AudioChannel` arayüzüne çağrılar.

## Oracle
Gerçek `SoundEngine`, kanalı yakalayan sahte `Channel` (`Library` yerine); satırlar çağrı dizisi (konum, perde, ses, mesafe, durdur).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Gerçek çıkış (T63), çözme (T62), oyun tetikleri (T64).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: üç paketin çözümü ve ses olayı tablosu (oracle'da gerçek codec'ler, `sound` ailesi), oturum `take_sounds`. Kayıt: `docs/client-sound-packets-parity-conformance.md`.
- Kalan: `SoundEngine.play/stop/tick` mantığı ve kanal parametreleri → [T61b](T61b-ses-motoru.md) (Channel gölgesiyle oracle).
