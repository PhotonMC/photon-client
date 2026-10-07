# Q02 — İngilizce README, katkı belgeleri, belge ayrımı

**Amaç:** Public ziyaretçinin ilk gördüğü yüzeyi İngilizce ve güncel yapmak; dilim kayıtlarını kamu belgelerinden ayırmak.
**Bağımlılık:** Q01. **Boy:** M.

## Oku
- [inceleme raporu](../code-quality-review-2026-10-07.md) §7
- `README.md`, `docs/index.md`, `tools/generate-doc-index.py`

## Yapılacaklar
- `README.md` İngilizce: ne, neden, durum (oynanabilirlik sınırları dürüstçe), ekran görüntüsü, derleme ve çalıştırma, asset politikası, resmî olmama notu.
- Türkçe README `docs/tr/README.md`'ye taşınır.
- `CONTRIBUTING.md` (derleme, test, oracle, commit biçimi: ≤72 karakter başlık), `SECURITY.md`.
- Dilim/parity kayıtları `docs/records/`'a taşınması için plan ve bağlantı güncelleyici betik; dizin üreticisi yeni düzene uyar.
- Q02b olarak çekirdek belgelerin (mimari, kodlama standartları, davranış testi) İngilizce sürümü ayrı görev satırı açılır.

## Kabul
Kırık iç bağlantı yok (`python3 tools/voxel.py check` belge adımları).

## Kapsam dışı
Dilim kayıtlarının çevirisi.

## Belgeler
Bu görev dosyasının devir notu; ölçüm sayıları yalnız burada.

## Devir notu
(boş)
