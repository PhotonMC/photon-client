# T61b — `SoundEngine` mantığı ve kanal parametreleri

**Amaç:** T61'in `SoundCommand`'larını `SoundEngine`'in yoluyla çalıştır: `play` (hacim, perde, sönüm mesafesi, göreli, döngü, sessiz başlama, kanal havuzu, durağan/akış), `stop`, `tick` (gecikmeli ve tikli sesler, silme zamanları, yönlü kaynaklar), kategori kazançları; kanalı bir arayüz (`AudioChannel`) arkasında tut (çıkış T63).
**Bağımlılık:** T61. **Boy:** L.

## Oku
- Java: `client/sounds/{SoundEngine,ChannelAccess,SoundEngineExecutor}.java`, `com/mojang/blaze3d/audio/{Channel,Library}.java`, `client/resources/sounds/{SoundInstance,AbstractSoundInstance,SimpleSoundInstance,EntityBoundSoundInstance}.java`, `client/Options.java` (`getFinalSoundSourceVolume`).
- Rust: `crates/voxel-game/src/sounds.rs` (kayıt defteri, paket tipleri).

## Oracle
Gerçek `SoundEngine`; **`Channel` ve `SoundBufferLibrary` test içinde gölgelenir** (`tests/behavior/java/org/lwjgl/system/MemoryUtil.java` örneği gibi, `tests/behavior/java/com/mojang/blaze3d/audio/Channel.java`: aynı genel yöntemler, çağrıları kaydeder; OpenAL yok). `SoundEngine`/`Library`/`Options` Unsafe ile kurulur. Satırlar: çağrı dizisi (konum, perde, ses, sönüm, döngü, oynat/durdur).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Gerçek çıkış (T63), çözme (T62), oyun tetikleri (T64).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
(boş)
