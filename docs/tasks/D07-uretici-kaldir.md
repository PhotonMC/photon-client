# D07 — Üreticiyi ve `generated/` dizinini kaldırma

**Amaç:** Üretim zincirini kapatmak ve kural metinlerini yeni modele uydurmak.
**Bağımlılık:** D02, D03, D04, D05, D06. **Boy:** S.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §6, §8
- `tools/generate/`, `crates/voxel-data/src/generated/`, `AGENTS.md`, `docs/coding-standards.md`

## Yapılacaklar
- `tools/generate/` ve `generated/` silinir; `voxel-data` modül yolları elle modüllere işaret eder.
- `AGENTS.md` "tablo oracle'dan yakalanır, üretilmiş Rust dosyasına yazılır" maddesi ve kodlama standartlarındaki "Referans verisi" bölümü: elle yazılmış Java karşılığı + her parity koşusunda tam karşılaştırma; sürüm atlamasında Java diff'ine göre elle güncelleme.
- Sürüm güncelleme rehberi (hangi Java dosyası hangi Rust modülü).

## Kabul
Davranış değişmez: `python3 tools/voxel.py check` → `CHECK OK` (parity dahil); çizime dokunduysa `capture_frame` görüntüsü öncekiyle piksel eşit.

## Kapsam dışı
–

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
