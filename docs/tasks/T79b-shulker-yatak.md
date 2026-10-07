# T79b — Shulker kutusu ve yatak çizimi

**Amaç:** T79'un kalanı: `ShulkerBoxRenderer` (kapak açısı, `ShulkerBoxBlockEntity` animasyonu, renkli dokular) ve yatak (`BedRenderer`, `BedRenderState`) blok entity çizimi.
**Bağımlılık:** T79. **Boy:** M.

## Oku
Java: `client/renderer/blockentity/{ShulkerBoxRenderer,BedRenderer}.java`, `client/renderer/blockentity/state/{ShulkerBoxRenderState,BedRenderState}.java`. Rust: `crates/voxel-render/src/chest.rs`, `crates/voxel-world/src/chest.rs`, `crates/voxel/src/chests.rs` (iskelet).

## Oracle
Gerçek renderer kaydı `entity/chest-pose` kalıbıyla (köşe, UV, ışık). Java kaynağı önce satır satır okunur.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Başka blok entity aileleri (T80–T82).

## Belgeler
Yeni contract/parity kaydı, `docs/index.md`.

## Devir notu
(boş)
