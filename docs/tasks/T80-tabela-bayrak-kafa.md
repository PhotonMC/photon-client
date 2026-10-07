# T80 — Tabela, bayrak, kafa ve çan blok entity çizimi

**Amaç:** `StandingSignRenderer`/`HangingSignRenderer` (metin, parıltı, balmumu), `BannerRenderer` (desen katmanları), `SkullBlockRenderer` (kafa, kafatası, oyuncu kafası profili), `BellRenderer` (sallanma).
**Bağımlılık:** T79. **Boy:** L.

## Oku
Java: `client/renderer/blockentity/{AbstractSignRenderer,StandingSignRenderer,HangingSignRenderer,BannerRenderer,SkullBlockRenderer,BellRenderer}.java` ve `state/` karşılıkları. Rust: T79, yazı için `crates/voxel-render/src/gui/font.rs`.

## Oracle
`block_entity_model` satırları; tabela metninin yerleşimi. Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
(boş)
