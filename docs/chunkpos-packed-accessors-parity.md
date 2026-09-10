# P01.c — ChunkPos packed koordinat erişimcileri doğrulaması

`src/binary.rs`, packed ChunkPos long içindeki düşük ve yüksek 32 bit alanlarını Java `ChunkPos.getX(long)` ve `getZ(long)` ile aynı signed i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API’ye bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, iki signed i32 sınır yerleşimi ve alternatif bit deseni doğrudan Java/Rust API’leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1683/1683** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **2185/2185** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2185 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `d91d4fb976af018c6ced8eb9d475396c09c64f7ef8429742661a9ba2d8c34267`. Birleşik referans kimliği: `595049f32d6822eff3379ec8f2d5211ef192043bf7f6a11db9b9d7c609fe01a1`.

Adapter hash’leri: debug `fe351e4afa245b39c89bece68a6dfdb02699933b5f980b5b82214942e923dc4d`, release `31dad93777713c698bdc325bea831bfa6833f276b66b5d5d665b91c0421b8bf6`.

Yerel raporlar: `compare-arjeniq4` (math debug), `compare-_0386q8q` (math release), `compare-63qzbune` (birleşik debug), `compare-1pb7x57b` (birleşik release) ve `verify-f_mau559` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından x/z çıkarma ve signed i32 yorumuyla sınırlıdır. Packed değer üreticileri, SectionPos dönüşümleri, stream codec tüketimi ve bu accessor’ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
