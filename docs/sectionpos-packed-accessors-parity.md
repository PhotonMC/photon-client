# P01.c — SectionPos packed koordinat erişimcileri doğrulaması

`src/binary.rs`, packed SectionPos long içindeki signed x, y ve z alanlarını Java `SectionPos.x(long)`, `y(long)` ve `z(long)` ile aynı i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API'lere bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, signed 22/20 bit uçları ve alternatif bit deseni doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1113/1113** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1615/1615** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1615 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `23d8549371228c5fe370021e1df1ce61a2d1869ad767cf1a029bcbf1930b5ac9`. Birleşik referans kimliği: `747c3645458af2ae3b9de41407461996409d550b8fca05e990e18b3b1983f461`.

Adapter hashleri: debug `f8fafbf2d2cd3ad40cd739a394f6d29f2575d4a9a3d7e85b6246579b4ce57682`, release `30554a139fc26f799697f0ab5c6ef3f2501d976b96dfa59d85ba7f2f9d7d6e96`.

Yerel raporlar: `compare-y28xdodh` (math debug), `compare-_7cfq3in` (math release), `compare-cxv1an3r` (birleşik debug), `compare-1jjs0aun` (birleşik release) ve `verify-71u5jh9x` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından section x/y/z çıkarma ve signed sign extension yorumuyla sınırlıdır. SectionPos üreticileri, blok dönüşümleri, relative konumlar, stream codec tüketimi ve bu accessor'ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
