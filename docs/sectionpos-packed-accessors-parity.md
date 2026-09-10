# P01.c — SectionPos packed koordinat erişimcileri doğrulaması

`src/binary.rs`, packed SectionPos long içindeki signed x, y ve z alanlarını Java `SectionPos.x(long)`, `y(long)` ve `z(long)` ile aynı i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API'lere bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, signed 22/20 bit uçları ve alternatif bit deseni doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1987/1987** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2489/2489** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2489 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `665bdda6827e71f559f4e9a959ed126f3d3e055594f3b677359c51d0d3a74e16`. Birleşik referans kimliği: `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc`.

Adapter hashleri: debug `f8fafbf2d2cd3ad40cd739a394f6d29f2575d4a9a3d7e85b6248979b4ce57682`, release `30554a139fc26f799697f0ab5c6ef3f2501d976b96dfa59d85ba7f2f9d7d6e96`.

Yerel raporlar: `compare-zt_h_egg` (math debug), `compare-7afcvx21` (math release), `compare-251aed0_` (birleşik debug), `compare-jqj2kta_` (birleşik release) ve `verify-csdh4b83` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından section x/y/z çıkarma ve signed sign extension yorumuyla sınırlıdır. SectionPos üreticileri, blok dönüşümleri, relative konumlar, stream codec tüketimi ve bu accessor'ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
