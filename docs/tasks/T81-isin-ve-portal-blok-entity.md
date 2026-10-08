# T81 — Işın, portal ve efektli blok entity'ler

**Amaç:** `BeaconRenderer` (ışın bölümleri), `TheEndPortalRenderer`/`TheEndGatewayRenderer`, `ConduitRenderer`, `EnchantTableRenderer` ve `LecternRenderer` (kitap modeli), `DecoratedPotRenderer`, `BrushableBlockRenderer`, `PistonHeadRenderer`, `ShelfRenderer`.
**Bağımlılık:** T79. **Boy:** M.

## Oku
Java: `client/renderer/blockentity/{BeaconRenderer,AbstractEndPortalRenderer,TheEndPortalRenderer,TheEndGatewayRenderer,ConduitRenderer,EnchantTableRenderer,LecternRenderer,DecoratedPotRenderer,BrushableBlockRenderer,PistonHeadRenderer,ShelfRenderer}.java`. Rust: T79.

## Oracle
`block_entity_model` satırları. Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/photon.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
(boş)
