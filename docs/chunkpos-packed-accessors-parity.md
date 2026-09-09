# P01.c — ChunkPos packed koordinat erişimcileri doğrulaması

`src/binary.rs`, packed ChunkPos long içindeki düşük ve yüksek 32 bit alanlarını Java `ChunkPos.getX(long)` ve `getZ(long)` ile aynı signed i32 sonuçlara çıkarır. Rust adapter ham packed hex girdisini bu API’ye bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos packed koordinat erişimcileri için 6 özgün senaryo hazırlandı.
- Sıfır, pozitif/negatif alanlar, iki signed i32 sınır yerleşimi ve alternatif bit deseni doğrudan Java/Rust API’leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1221/1221** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **1723/1723** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1723 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `df8b0fd452c7f12175fe9ced3c534c1f64bdb7df75cc28529becd0eb8ab97b3c`. Birleşik referans kimliği: `86380f621c6f188dddfa1e68a94057a8ada6d638b9f5c04d3a2c9682b1ca75cf`.

Adapter hash’leri: debug `fe351e4afa245b39c89bece68a6dfdb02699933b5f980b5b82214942e923dc4d`, release `31dad93777713c698bdc325bea831bfa6833f276b66b5d5d665b91c0421b8bf6`.

Yerel raporlar: `compare-6dvmyv_h` (math debug), `compare-ak0n9_u3` (math release), `compare-_8esk0jy` (birleşik debug), `compare-g98k9sn0` (birleşik release) ve `verify-72fv37y0` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, ham packed long alanlarından x/z çıkarma ve signed i32 yorumuyla sınırlıdır. Packed değer üreticileri, SectionPos dönüşümleri, stream codec tüketimi ve bu accessor’ların daha geniş istemci akışlarındaki kullanımı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
