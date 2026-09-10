# P01.c — ChunkPos packed koordinat erişimcileri doğrulaması

`src/binary.rs`, packed ChunkPos long içindeki düşük ve yüksek 32 bit alanlarını Java `ChunkPos.getX(long)` ve `getZ(long)` ile aynı signed i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API’ye bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, iki signed i32 sınır yerleşimi ve alternatif bit deseni doğrudan Java/Rust API’leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1987/1987** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **2489/2489** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2489 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `665bdda6827e71f559f4e9a959ed126f3d3e055594f3b677359c51d0d3a74e16`. Birleşik referans kimliği: `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc`.

Adapter hash’leri: debug `fe351e4afa245b39c89bece68a6dfdb02699933b5f980b5b82214942e923dc4d`, release `31dad93777713c698bdc325bea831bfa6833f276b66b5d5d665b91c0421b8bf6`.

Yerel raporlar: `compare-zt_h_egg` (math debug), `compare-7afcvx21` (math release), `compare-251aed0_` (birleşik debug), `compare-jqj2kta_` (birleşik release) ve `verify-csdh4b83` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından x/z çıkarma ve signed i32 yorumuyla sınırlıdır. Packed değer üreticileri, SectionPos dönüşümleri, stream codec tüketimi ve bu accessor’ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
