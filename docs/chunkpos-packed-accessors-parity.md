# P01.c — ChunkPos packed koordinat erişimcileri doğrulaması

`src/binary.rs`, packed ChunkPos long içindeki düşük ve yüksek 32 bit alanlarını Java `ChunkPos.getX(long)` ve `getZ(long)` ile aynı signed i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API’ye bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, iki signed i32 sınır yerleşimi ve alternatif bit deseni doğrudan Java/Rust API’leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1319/1319** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **1821/1821** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1821 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `85b9ce09336ec312c84a062e772cdb41b6a2e079b6fcb1995200470edf61dd6c`. Birleşik referans kimliği: `01a770901948bb0f5650c8b4e80bf54b513d904c9746232c38b93c03a7546957`.

Adapter hash’leri: debug `fe351e4afa245b39c89bece68a6dfdb02699933b5f980b5b82214942e923dc4d`, release `31dad93777713c698bdc325bea831bfa6833f276b66b5d5d665b91c0421b8bf6`.

Yerel raporlar: `compare-stkj42d2` (math debug), `compare-e9upg098` (math release), `compare-olmqgxpc` (birleşik debug), `compare-jxg0__tj` (birleşik release) ve `verify-m0natom3` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından x/z çıkarma ve signed i32 yorumuyla sınırlıdır. Packed değer üreticileri, SectionPos dönüşümleri, stream codec tüketimi ve bu accessor’ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
