# P01.c — ChunkPos packed koordinat erişimcileri doğrulaması

`src/binary.rs`, packed ChunkPos long içindeki düşük ve yüksek 32 bit alanlarını Java `ChunkPos.getX(long)` ve `getZ(long)` ile aynı signed i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API’ye bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, iki signed i32 sınır yerleşimi ve alternatif bit deseni doğrudan Java/Rust API’leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1653/1653** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **2155/2155** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2155 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `e9b1c1e55b2a03528b712cd304101c7b36378556372ebad60bb330a4a60eb192`. Birleşik referans kimliği: `2a68356f797e134e8a268355635fffb2db6a83b94e11d3afbdf308978093028e`.

Adapter hash’leri: debug `fe351e4afa245b39c89bece68a6dfdb02699933b5f980b5b82214942e923dc4d`, release `31dad93777713c698bdc325bea831bfa6833f276b66b5d5d665b91c0421b8bf6`.

Yerel raporlar: `compare-ceusm4y4` (math debug), `compare-hconpy1a` (math release), `compare-rhjx0z7e` (birleşik debug), `compare-jtl411eu` (birleşik release) ve `verify-u5d5zd4c` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından x/z çıkarma ve signed i32 yorumuyla sınırlıdır. Packed değer üreticileri, SectionPos dönüşümleri, stream codec tüketimi ve bu accessor’ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
