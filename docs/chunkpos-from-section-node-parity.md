# P01.c — ChunkPos section node dönüşümü doğrulaması

`src/binary.rs`, Java `ChunkPos.fromSectionNode(long)` yolunu SectionPos packed long’undan signed x/z alanlarını çıkarıp ChunkPos packed long’una dönüştürerek karşılar. Rust adaptörü sonucu Java ile aynı unsigned lowercase hex biçiminde taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos section node dönüşümü için 6 özgün senaryo hazırlandı.
- Pozitif/negatif section koordinatları, signed 22 bit minimum/maksimum uçları ve y alanının atılması eşleşti.
- Rust debug ve release adaptörleri güncel **1478/1478** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **1980/1980** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1980 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `abe3d8cdf014db1ee9b2fa953ff1b7ba7bc40e31c9d35672c0cd8adb20896f50`. Birleşik referans kimliği: `e068e38cc1d0ab64ac2844f20d474f313e2325b42cc620af39051ca8aa6748cd`.

Yerel raporlar: `compare-xrmyz06w` (math debug), `compare-zrxwzjp_` (math release), `compare-dxdmken0` (birleşik debug), `compare-cvif6gdr` (birleşik release) ve `verify-jlkp_8wf` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos.fromSectionNode(long)` dönüşümünün x/z çıkarma, y atma ve packed long yerleşimini kapsar. SectionPos üreticileri, node offset işlemleri, BlockPos dönüşümleri, `ChunkPos.rangeClosed`, chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
