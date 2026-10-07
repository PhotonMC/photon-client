# T79 — Blok entity çizici iskeleti ve sandık ailesi

**Amaç:** `BlockEntityRenderDispatcher` (blok entity verisi → çizim durumu → model), sandık/tuzaklı/ender sandığı (`ChestRenderer`: kapak açısı, çift sandık), shulker kutusu (`ShulkerBoxRenderer`), yatak (`BedRenderState`).
**Bağımlılık:** –. **Boy:** L.

## Oku
Java: `client/renderer/blockentity/{BlockEntityRenderDispatcher,BlockEntityRenderer,ChestRenderer,ShulkerBoxRenderer}.java`, `client/renderer/blockentity/state/{ChestRenderState,ShulkerBoxRenderState,BedRenderState}.java`, `client/renderer/special/{ChestSpecialRenderer,ShulkerBoxSpecialRenderer}.java`. Rust: `docs/records/client-world-block-entity-conformance.md` (veri), `crates/voxel-render/src/{mob_model,special_models,skull}.rs` (model katmanı tabloları).

## Oracle
Gerçek `ChestRenderer.submit` kaydı (köşe, UV, ışık) `block_entity_model` operation'ında. Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
Yapılan: sandık ailesi (tek/çift, tuzaklı, ender, bakır) `ChestRenderer`'a bit bit eşit; kapak `BlockEvent` ile sürülür; istemci sandığı blok durumundan kurar. Kayıt: [client-chest-parity-conformance.md](../records/client-chest-parity-conformance.md).
Sapma: Noel dokusu yok (tarih denetimi). Kapak hareketi canlıda ayrıca denenmedi.
Açık: shulker kutusu ve yatak [T79b](T79b-shulker-yatak.md)'ye; sandık sesleri T78'de.
