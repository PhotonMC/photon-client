# T62 — Ogg/Vorbis çözümü (`lewton`)

**Amaç:** `.ogg` ses nesnelerini 16 bit işaretli PCM'e çöz; referansın `JOrbisAudioStream` çıktısıyla örnek örnek karşılaştır. Yeni üretim bağımlılığı: `lewton` (kararlı sürüm 0.10.2).
**Bağımlılık:** yok (T60/T61'den bağımsız). Ses yığını kararı: [`audio-backend-decision.md`](../audio-backend-decision.md) (çıkış `cpal`, çözücü `lewton`).

## Oku
- Java: `client/sounds/{JOrbisAudioStream,AudioStream,FloatSampleSource,FiniteAudioStream,LoopingAudioStream,ChunkedSampleByteBuf}.java`, `com/mojang/blaze3d/audio/SoundBuffer.java`.
- Rust: kök `Cargo.toml` (`[workspace.dependencies]`), `crates/voxel-game` ya da yeni `voxel-audio` crate'i (katman kuralı: `docs/coding-standards.md`).

## Yapılacaklar
1. `lewton` ile çözücü; kanal sayısı, örnekleme hızı, toplam örnek; float→16 bit dönüşümü JOrbis'inkiyle (yuvarlama/kırpma) aynı.
2. Döngü ve akış parçalama (`ChunkedSampleByteBuf`) mantığı.
3. Fark çıkarsa `symphonia` denenir; ikisi de tutmazsa JOrbis portu kararı kullanıcıya sorulur.

## Oracle
Gerçek `JOrbisAudioStream` ile birkaç gerçek ses nesnesi (kısa, uzun, mono, stereo); satırlar PCM bayt sağlaması (SHA-256) ve ilk/son örnekler.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Çalma (T63), ses dosyalarını Git'e koymak (nesneler `.data/`da kalır).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: yeni `voxel-audio` crate'i (`lewton` 0.10.2), `OggStream`/`LoopingOggStream`/`decode_ogg`, `sound/decode` oracle'ı (gerçek `JOrbisAudioStream`; örnekler yerelde `reference-pcm/`). Kayıt: `docs/records/client-ogg-decoding-parity-conformance.md`.
- Sapma: bit eşitliği yok, her örnek ±1 içinde (%0,045 farklı); JOrbis portu kararı gerekçe doğarsa ayrı görev.
