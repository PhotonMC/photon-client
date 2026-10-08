# D02 — Animasyonlar: Java sınıfı başına elle modül

**Amaç:** `generated/animations.rs`'yi Java'nın animasyon sınıflarına karşılık gelen modüllere çevirmek.
**Bağımlılık:** D01. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §6
- D01 envanteri, `crates/photon-data/src/{generated/animations.rs,animation.rs}`

## Yapılacaklar
- Tek seferlik dönüşüm betikle yapılabilir; çıktı elle bakılan kaynaktır (başlıkta "generated" yazmaz).
- Java sınıfı başına modül, okunur oluşturucu (`AnimationDefinition::builder(..).add_animation(..)`).
- Parity karşılaştırması aynen doğrular; üretici betiğin bu tabloya ait kısmı silinir.

## Kabul
Davranış değişmez: `python3 tools/photon.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
Animasyon oynatıcı değişikliği.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
