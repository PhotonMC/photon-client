# P01.c — SectionPos packed koordinat erişimcileri doğrulaması

`src/binary.rs`, packed SectionPos long içindeki signed x, y ve z alanlarını Java `SectionPos.x(long)`, `y(long)` ve `z(long)` ile aynı i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API'lere bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, signed 22/20 bit uçları ve alternatif bit deseni doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1301/1301** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1803/1803** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1803 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `a7681fdbc73c8faa861a3c0f74be7f2286257b8c494b6f1d3c85598ad65f6973`. Birleşik referans kimliği: `e26e5fb4cdc24beb08ca189d4d55904fcff008d93a9a99a606f787f60554f935`.

Adapter hashleri: debug `f8fafbf2d2cd3ad40cd739a394f6d29f2575d4a9a3d7e85b6246579b4ce57682`, release `30554a139fc26f799697f0ab5c6ef3f2501d976b96dfa59d85ba7f2f9d7d6e96`.

Yerel raporlar: `compare-zq98bs3k` (math debug), `compare-zgg863t0` (math release), `compare-yz31bmcf` (birleşik debug), `compare-90dzj4qp` (birleşik release) ve `verify-l29xakrv` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından section x/y/z çıkarma ve signed sign extension yorumuyla sınırlıdır. SectionPos üreticileri, blok dönüşümleri, relative konumlar, stream codec tüketimi ve bu accessor'ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
