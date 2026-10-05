# T37h3 — Ender ejderhasının kristal ışını

**Amaç:** `EnderCrystal` varlığı (çizim: çekirdek ve çerçeveler, `EndCrystalRenderer.getY` sıçraması) ve ejderhanın `nearestCrystal` seçimi (`checkCrystals`, 32 blok, rastgele), `submitCrystalBeams` (`endCrystalBeam`) ve kristalin kendi ışını (`BeamTarget`).
**Bağımlılık:** T37h. **Boy:** M.

## Oku
- Java: `world/entity/boss/enderdragon/{EnderDragon,EndCrystal}.java`, `client/renderer/entity/{EnderDragonRenderer,EndCrystalRenderer}.java`, `client/model/object/crystal/EndCrystalModel.java`.
- Rust: `crates/voxel/src/ender_dragons.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
