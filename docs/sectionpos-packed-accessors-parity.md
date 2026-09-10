# P01.c — SectionPos packed koordinat erişimcileri doğrulaması

`src/binary.rs`, packed SectionPos long içindeki signed x, y ve z alanlarını Java `SectionPos.x(long)`, `y(long)` ve `z(long)` ile aynı i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API'lere bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, signed 22/20 bit uçları ve alternatif bit deseni doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2023/2023** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2525/2525** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2525 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `1a217debf64171ea8f99e220ffd6aca1130ecceef3e6015b82f0c12a4e3f9f0d`. Birleşik referans kimliği: `38386c8d844bb2560f83f62571433316aca08b176233a98979cccf45ec22a8d5`.

Adapter hashleri: debug `f8fafbf2d2cd3ad40cd739a394f6d29f2575d4a9a3d7e85b6252579b4ce57682`, release `30554a139fc26f799697f0ab5c6ef3f2525d976b96dfa59d85ba7f2f9d7d6e96`.

Yerel raporlar: `compare-4sx1lm2m` (math debug), `compare-m3ib6sjn` (math release), `compare-czvhou3w` (birleşik debug), `compare-oz19j96e` (birleşik release) ve `verify-nukwd4yc` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından section x/y/z çıkarma ve signed sign extension yorumuyla sınırlıdır. SectionPos üreticileri, blok dönüşümleri, relative konumlar, stream codec tüketimi ve bu accessor'ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
