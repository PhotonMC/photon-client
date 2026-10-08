# Q02b — Çekirdek belgelerin İngilizce sürümü

**Amaç:** Public katkıcının okuyacağı çekirdek belgeleri İngilizce sunmak; dilim kayıtları Türkçe kalır.
**Bağımlılık:** Q02. **Boy:** M.

## Oku
- [Q02 devir notu](Q02-ingilizce-readme.md)
- `docs/coding-standards.md`, `docs/behavior-testing.md`, `docs/behavior-runner.md`, `docs/local-setup.md`, `AGENTS.md`

## Yapılacaklar
- Dört belgenin İngilizce sürümü (kodlama standartları, davranış testi, runner, yerel kurulum). Çeviri `docs/` altında aynı adla, Türkçe asıl `docs/tr/` altına taşınır; ya da tersi: karar kullanıcıya sorulur.
- `AGENTS.md` kuralları aynı kalır; yalnız `CONTRIBUTING.md` ve README'nin Türkçe işaret ettiği bağlantılar İngilizce sürüme çevrilir.
- Dizin üreticisi ve `check-doc-links.py` yeni düzene uyar.

## Kabul
`python3 tools/photon.py check` belge adımları yeşil; README ve CONTRIBUTING'deki "(Turkish)" notları kalkar.

## Kapsam dışı
Dilim kayıtlarının (`docs/records/`) ve görev dosyalarının çevirisi.

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
- `docs/en/` altında dört çeviri: `coding-standards.md`, `behavior-testing.md`, `behavior-runner.md`, `local-setup.md`. Her biri başında Türkçe aslın geçerli olduğunu söyler; asıl dosyalar yerinde kaldı (taşıma yok, böylece kayıt bağlantıları ve AGENTS.md yolları bozulmadı). Bağlantılar çeviri belgeleri arasında İngilizceye, diğer belgelere Türkçe asla gider.
- README ve CONTRIBUTING İngilizce sürümlere bağlandı; "(Turkish)" işaretleri kalktı. Belge dizini üreticisi `docs/en/` için ayrı bir bölüm yazar.
- Çeviri ve asıl arasında sürüklenme riski var: Türkçe asıl değişince çeviri elle güncellenir (otomatik denetim yok). `local-setup.md`taki eski `libshaderc_shared` notu, V02 ile ikili shaderc'e bağlanmadığı için iki dilde de güncellendi (`otool -L` ile doğrulandı).
