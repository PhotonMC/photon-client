# T71b — Blok kırıntılarının çizimi, tintler ve oyuna bağlama

**Amaç:** `TerrainMaterials`in gerçeği (bloğun parçacık dokusu model setinden, `colorAsTerrainParticle` tint kaynaklarından, biyom tintleri dahil), kırıntıların blok atlasından çizimi (`ParticleAtlas::Blocks`, `sprite_window`), `LevelEvent 2001`in kırıntıya bağlanması (`take_block_breaks`) ve `MultiPlayerGameMode.continueDestroyBlock`un çatlak kırıntısı (`add_breaking_block_effect`).
**Bağımlılık:** T71. **Boy:** M.

## Oku
Java: `client/color/block/{BlockTintSources,BlockColors}.java`, `client/multiplayer/MultiPlayerGameMode.java` (`continueDestroyBlock`), `client/renderer/block/BlockStateModelSet.java` (`getParticleMaterial`). Rust: `crates/voxel-world/src/particles/terrain.rs`, `crates/voxel-render/src/block/biome_tint.rs`, `crates/voxel-data/src/generated/block/tints.rs`, `crates/voxel/src/particles.rs`, `crates/voxel-game/src/game_mode.rs`.

## Oracle
Gerçek `BlockColors` ile biyoma bağlı tintler için oracle (renk haritaları yüklenmiş sahne) ya da kayıtta açık sapma; çizim `particle_quads`ın blok atlasındaki dörtgeni (`client-parity`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`; bir blok kırılınca kırıntılar ve kırarken yüzden çatlak kırıntısı canlıda görülür (`shot`, bir kez).

## Kapsam dışı
`block_marker`, `falling_dust`; başka partikül aileleri.

## Belgeler
Bu görevin kaydı ve `docs/index.md`.

## Devir notu
- **Yapılan:** `BlockRenderer::{particle_sprite,particle_tint,sprite_rect}`; `DrawKind::BlockParticles`/`Run::BlockParticle` (blok atlası, arazi boru hattı); uygulamada `BlockMaterials` (`TerrainMaterials`), kırılma olayı 2001 ve `continue_attack`ın çatlak isteği (`take_block_breaks`, `take_breaking_effects`). Canlı: taş ve yaprak kırıntıları görüldü.
- **Açık:** modelin parçacık dokusu ve tintin Java'yla bire bir karşılaştırması (oracle yok), su/kabarcık tinti, `missingno` yedeği.
