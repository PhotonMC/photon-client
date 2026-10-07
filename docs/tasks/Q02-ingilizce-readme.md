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
- Kök `README.md` İngilizce yazıldı; eski Türkçe README `docs/tr/README.md`'ye taşındı (bağlantıları `../` ile düzeltildi). Ekran görüntüsü bilerek yok: her kare Mojang asset'i içerir, depoya girmez.
- `CONTRIBUTING.md` ve `SECURITY.md` eklendi. SECURITY GitHub özel güvenlik bildirimine dayanır; depo sahibi bunu Security sekmesinden açmalı (e-posta uydurulmadı).
- `tools/move-records.py` 641 kaydı (`*-conformance.md` 632, `*-record.md` 9) `docs/records/`'a taşıdı; Markdown bağlantılarını eski konuma göre çözüp yeniden yazdı, `docs/<ad>.md` metin anmalarını (kod yorumu, AGENTS, görevler) güncelledi. Betik idempotent; yeniden çalıştırmak "nothing to move" der.
- `tools/check-doc-links.py` yeni: göreli Markdown bağlantılarını denetler, kapıya (`voxel.py check`) bağlandı. Öncesinde iki kırık bağlantı vardı (`chunkpos`, `game-connection`), düzeltildi.
- Dizin üreticisi `docs/records/`'ı da tarıyor.
- Açık: belgelerin çoğu hâlâ Türkçe (Q02b); README'deki yerel kurulum bağlantısı "(Turkish)" diye işaretli.
