# T37g — Wither

**Amaç:** `WitherBossModel` (üç baş, kuyruk, omurga), `WitherBossRenderer` (hasarda mavi/beyaz kaplama, zırh kabuğu), `WitherArmorLayer` (enerji girdabı), `WitherBoss` istemci durumu (`DATA_TARGET_*`, `DATA_ID_INV`, kalkan fazı).
**Bağımlılık:** T37, T33d (enerji girdabı). **Boy:** M.

## Oku
- Java: `world/entity/boss/wither/WitherBoss.java`, `client/model/monster/wither/WitherBossModel.java`, `client/renderer/entity/{WitherBossRenderer,layers/WitherArmorLayer,state/WitherRenderState}.java`.
- Rust: `crates/voxel/src/mobs.rs` (enerji girdabı kalıbı: şarjlı creeper), `crates/voxel-render/src/mob_model.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
