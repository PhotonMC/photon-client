# P01.c — SectionPos packed koordinat erişimcileri doğrulaması

`src/binary.rs`, packed SectionPos long içindeki signed x, y ve z alanlarını Java `SectionPos.x(long)`, `y(long)` ve `z(long)` ile aynı i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API'lere bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, signed 22/20 bit uçları ve alternatif bit deseni doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1007/1007** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1509/1509** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1509 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `7b8b9ea0f5717fc78d7092f1d3563134bfdae91e1c70cf411bb3269f1ad58d4d`. Birleşik referans kimliği: `f71b676b2ae17505734ec3d87f208055714fa5dd6b9cc9417d155abf92e2ac0a`.

Adapter hashleri: debug `f8fafbf2d2cd3ad40cd739a394f6d29f2575d4a9a3d7e85b6246579b4ce57682`, release `30554a139fc26f799697f0ab5c6ef3f2501d976b96dfa59d85ba7f2f9d7d6e96`.

Yerel raporlar: `compare-upfzoh9c` (math debug), `compare-czpd1pgo` (math release), `compare-_3h0xf61` (birleşik debug), `compare-a2h2wcm2` (birleşik release) ve `verify-19j4rwux` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından section x/y/z çıkarma ve signed sign extension yorumuyla sınırlıdır. SectionPos üreticileri, blok dönüşümleri, relative konumlar, stream codec tüketimi ve bu accessor'ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
