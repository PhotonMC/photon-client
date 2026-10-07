# A06b — Tek dev işlev olan dosyalar ve oracle adaptörünün crate'i

**Amaç:** A06'nın bölünemeyen kalanı: 600 satırı aşan işlevleri/dosyaları işlev başına bölmek.
**Bağımlılık:** A06. **Boy:** L.

## Oku
- [A06 devir notu](A06-dev-dosyalar.md) (kalan dosyaların listesi), [dosya boyutu kuralı](../coding-standards.md#dosya-boyutu)

## Yapılacaklar
- Adaptörde `match operation`/`if operation == …` kolları işlev başına (`*_result`) çıkarılır, dağıtım bir `(ad, işlev)` tablosuna iner; `math_cases.rs`, `shape.rs`, `ticks_2.rs`, `gui/result.rs`, `entity.rs`.
- `probe/mode.rs`: test izleme (A04 `Recorder`) ile birlikte aşamalara; `play/incoming`: paket başına işleyici (A04).
- `outline.rs`: tek işlev, bloğa göre alt işlevler.
- Python: `math_cases` üreticisi konuya göre alt üreticilere; `cases.py` bir pakete (`cases/__init__.py` + konu modülleri), vaka listesinin bayt eşitliği önce/sonra hash'iyle doğrulanır.
- Java oracle sınıfları statik yöntem gruplarına göre sınıf zincirine (veya ayrı sınıflara) bölünür; runner derlemesi ve `check --capture` ile doğrulanır.
- `crates/voxel-oracle` (publish=false) crate'i: adaptör `examples/`'tan taşınır.

## Kabul
Hiçbir kod dosyası 600 satırı aşmaz; `python3 tools/voxel.py check --capture` → `CHECK OK`; vaka listesi ve Java çıktıları önceki capture ile eşit.

## Kapsam dışı
Davranış değişikliği; üretilmiş tablolar (D görevleri).

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
(boş)
