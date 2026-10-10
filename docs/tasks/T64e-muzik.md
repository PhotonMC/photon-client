# T64e — Müzik

**Amaç:** `MusicManager` ve oyundaki müzik ([T64d](T64d-ses-tetikleri-ortam-muzik.md)'den ayrıldı).
**Bağımlılık:** T64d. **Boy:** M.

## Devir notu
Yapıldı: `photon-game/sounds/music.rs` (yönetici, `BackgroundMusic.select`), `photon/music.rs` (biyom/boyut özniteliği, ev sahibi), `SoundSystem::play_music`; oracle `gui/music-manager` eşit. Kayıt: [client-music-parity-conformance.md](../records/client-music-parity-conformance.md). Açık: ekran müziği, ejderha müziği, bildirim.
