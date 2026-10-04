# T36c — Papağan, breeze ve mutlu ghast

**Amaç:** `ParrotModel` (konumlar: ayakta, uçan, oturan, omuzda; kanat çırpma durumu, dans), `BreezeModel` + `BreezeEyesLayer` + `BreezeWindLayer`, `HappyGhastModel` (+ koşum ve ip katmanı).
**Bağımlılık:** T36. **Boy:** L (sığmazsa böl).

## Oku
- Java: `client/model/animal/parrot/ParrotModel.java`, `client/model/monster/breeze/BreezeModel.java`, `client/model/animal/ghast/HappyGhast*.java`, `client/renderer/entity/{Parrot,Breeze,HappyGhast}Renderer.java`, `client/renderer/entity/layers/{Breeze*,ParrotOnShoulder}Layer.java`, `world/entity/animal/parrot/Parrot.java` (`calculateFlapping`).
- Rust: `crates/voxel/src/monsters.rs`, `crates/voxel-render/src/mob_model.rs`.

## Oracle
`airborne_model` kalıbı; gerçek varlık tiki için `living-entity-animations`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar, omuz papağanı (oyuncu katmanı), davranışlar.

## Devir notu
(boş)
