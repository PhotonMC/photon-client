# D05 — Eşyalar, bileşen türleri ve yaratıcı sekmeleri elle

**Amaç:** `generated/item/*` dosyalarını `Items`, `DataComponents`, `CreativeModeTabs` karşılıklarına çevirmek.
**Bağımlılık:** D01. **Boy:** L.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §6
- D01 envanteri, `crates/photon-data/src/{generated/item,item}/`

## Yapılacaklar
- `item/items.rs` Java'daki sırayla; `creative_tabs` Java'nın sekme kurulum mantığı hesaplanabiliyorsa (c), değilse (a).
- İksir/etki/öznitelik tabloları envantere göre.

## Kabul
Davranış değişmez: `python3 tools/photon.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Eşya davranışı.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
