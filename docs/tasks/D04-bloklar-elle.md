# D04 — Bloklar ve blok sınıfları elle

**Amaç:** `generated/block/*` dosyalarını `Blocks`/blok sınıfı/özellik karşılıklarına çevirmek.
**Bağımlılık:** D01. **Boy:** L.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §6
- D01 envanteri, `crates/photon-data/src/{generated/block,block}/`

## Yapılacaklar
- `block/blocks.rs` Java'daki sırayla elle tanımlar (mevcut `Block::of` DSL'i korunur); `block/properties.rs` (`BlockStateProperties`), sınıflar.
- Ses türleri (`block_sounds`) ve renklendirme (`tints`) envantere göre (a) veya (c).
- Parity karşılaştırması bütün blok/durum kayıt defterini doğrular.

## Kabul
Davranış değişmez: `python3 tools/photon.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Blok davranışı.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
