# T36b — Yarasa, vex ve allay

**Amaç:** `BatModel` (asılı ve uçan, `BatRenderer`), `VexModel` (kanatlar, yüklenme, ışıklı doku), `AllayModel` (kanatlar, dans, elindeki eşya) ve ilgili istemci durumları (`Bat.isResting`, `Vex` bayrakları, `Allay` dans ve ses verisi).
**Bağımlılık:** T36. **Boy:** M–L.

## Oku
- Java: `client/model/ambient/BatModel.java`, `client/model/animal/allay/AllayModel.java`, `client/model/monster/vex/VexModel.java`, `client/renderer/entity/{Bat,Vex,Allay}Renderer.java` ve durum sınıfları, `world/entity/ambient/Bat.java`, `world/entity/monster/Vex.java`, `world/entity/animal/allay/Allay.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`setup_phantom`, `setup_ghast`: yaşa bağlı kanat kalıbı), `crates/voxel/src/monsters.rs` (`MONSTERS` tablosu, `pose_monster`).

## Oracle
`airborne_model` kalıbı (gerçek model + durum); gerçek varlık tiki için `living-entity-animations`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar, davranışlar.

## Devir notu
(boş)
