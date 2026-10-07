# T79b — Shulker kutusu çizimi

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
Yapılan: shulker kutusu (`ShulkerBoxRenderer`: yön, renk, kapak ilerlemesi) gerçek renderer'a bit bit eşit; kapak `BlockEvent` 1 ile sürülür. Kayıt: [client-chest-parity-conformance.md](../records/client-chest-parity-conformance.md).
Java'dan sapma: yatak için iş yok (26.2'de blok modeli); kapak açılırken komşu güncellemesi ve varlık itme yok.
Açık: kapak hareketi canlıda denenmedi.
