# T37h — Ender ejderhası

**Amaç:** `EnderDragonModel` (parçalı gövde, kanat ve boyun dizileri), `EnderDragonRenderer` (uçuş geçmişi `DragonFlightHistory`, ölüm ışını, kristal ışını), `EnderDragon` istemci durumu ve `EnderDragonPart` seçilebilirliği (T26 notu).
**Bağımlılık:** T37. **Boy:** L.

## Oku
- Java: `world/entity/boss/enderdragon/{EnderDragon,EnderDragonPart}.java`, `client/model/monster/dragon/EnderDragonModel.java`, `client/renderer/entity/{EnderDragonRenderer,state/EnderDragonRenderState}.java`.
- Rust: `crates/voxel-world/src/pick.rs` (seçilebilirlik), `crates/voxel-render/src/mob_model.rs`.

## Kapsam dışı
Savaş yapıları, ejderha yumurtası, ses.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
