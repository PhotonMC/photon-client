# T80d — Tabela bloğu (ayakta ve asılı)

**Amaç:** `StandingSignRenderer`, `HangingSignRenderer`, `AbstractSignRenderer`: model, ahşap dokusu, metin yerleşimi (iki yüz), parıltı, balmumu; `BlockEntityData` ile metin.
**Bağımlılık:** T80. **Boy:** L.

## Oku
Java: `client/renderer/blockentity/{AbstractSignRenderer,StandingSignRenderer,HangingSignRenderer}.java`, `state/*SignRenderState.java`. Rust: `crates/voxel-render/src/gui/font.rs`, `crates/voxel/src/block_entities.rs`.

## Oracle
Gerçek renderer kaydı `entity/chest-pose` kalıbıyla (poz, ışık, doku, dörtgenler). Java kaynağı önce satır satır okunur.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Başka blok entity aileleri.

## Belgeler
`docs/client-chest-parity-conformance.md`'ye bölüm, `docs/index.md` gerekirse.

## Devir notu
(boş)
