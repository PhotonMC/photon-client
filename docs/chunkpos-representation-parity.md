# P01.c — ChunkPos temsil ve hash doğrulaması

`src/binary.rs`, Java `ChunkPos.toString()`, `hashCode()` ve static `hash` davranışlarını aynı metin ve i32 wrapping aritmetiğiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik signed koordinat girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Temsil ve hash için 6 özgün senaryo hazırlandı.
- `[x, z]` metin biçimi, instance/static hash yolu, signed sınırlar ve çarpma/toplama overflow davranışı Java i32 sonucuyla eşleşti.
- Rust debug ve release adaptörleri güncel **1783/1783** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2285/2285** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2285 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `c9c0c64d34c86e78a480fb2f63bd3bd5633a857332ad5997d97039c193be5338`. Birleşik referans kimliği: `6251a239b5efd4862d79c5b740dcb208ec9e1932f8a04d0d86c05a044353fd90`.

Yerel raporlar: `compare-1g27oorn` (math debug), `compare-bkcvw7oi` (math release), `compare-qmmqnzwk` (birleşik debug), `compare-ogma6668` (birleşik release) ve `verify-1hv399w4` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` metin temsilini ve iki hash yolunu kapsar. Record equality, packing/codec'ler, region/block/BlockPos yardımcıları, range/distance ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
