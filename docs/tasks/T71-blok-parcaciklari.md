# T71 — Blok parçacıkları: kırılma ve çatlama (`TerrainParticle`)

**Amaç:** `TerrainParticle`, `ParticleEngine.destroy` (bloğun kutucuklara bölünmesi) ve `crack` (kırarken yüz parçacıkları), `BlockParticleOption` paketi, `LevelEvent 2001` ve `MultiPlayerGameMode.continueDestroyBlock`'a bağlama.
**Bağımlılık:** T70. **Boy:** M.

## Oku
Java: `client/particle/{TerrainParticle,ParticleEngine}.java`, `core/particles/BlockParticleOption.java`, `client/renderer/LevelEventHandler.java` (2001), `client/multiplayer/MultiPlayerGameMode.java` (`continueDestroyBlock`). Rust: `crates/voxel-world/src/particles/`, `crates/voxel-game/src/game_mode.rs`.

## Oracle
`particle_state` operation'ı: kutucuk konumları, hızları (rastgele sırasıyla), doku seçimi, ışık; `client-parity`. Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
- **Yapılan:** `particles/terrain.rs` (`Particle::terrain`, üç sağlayıcı, `destroy_block_effect`, `breaking_block_effect`, `TerrainMaterials`); motor imzalarına `world` ve blok durumu; `LevelParticles.block_state` (paket `BlockParticleOption`), tür tablosunda `block_state`; dünyada `take_block_breaks` (olay 2001); oracle `particle_terrain`; kayıt: `client-particle-terrain-parity-conformance.md`.
- **Bölündü:** gerçek malzeme/tint çözümü, blok atlasından çizim, oyuna bağlama T71b. Açık: biyoma bağlı tintler oracle'da yok.
