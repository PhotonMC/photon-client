# P01.c — ChunkPos temsil ve hash doğrulaması

`src/binary.rs`, Java `ChunkPos.toString()`, `hashCode()` ve static `hash` davranışlarını aynı metin ve i32 wrapping aritmetiğiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik signed koordinat girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Temsil ve hash için 6 özgün senaryo hazırlandı.
- `[x, z]` metin biçimi, instance/static hash yolu, signed sınırlar ve çarpma/toplama overflow davranışı Java i32 sonucuyla eşleşti.
- Rust debug ve release adaptörleri güncel **1478/1478** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1980/1980** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1980 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `abe3d8cdf014db1ee9b2fa953ff1b7ba7bc40e31c9d35672c0cd8adb20896f50`. Birleşik referans kimliği: `e068e38cc1d0ab64ac2844f20d474f313e2325b42cc620af39051ca8aa6748cd`.

Yerel raporlar: `compare-xrmyz06w` (math debug), `compare-zrxwzjp_` (math release), `compare-dxdmken0` (birleşik debug), `compare-cvif6gdr` (birleşik release) ve `verify-jlkp_8wf` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos` metin temsilini ve iki hash yolunu kapsar. Record equality, packing/codec'ler, region/block/BlockPos yardımcıları, range/distance ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
