# P01.c — ChunkPos public sabitleri doğrulaması

`src/binary.rs`, Java `ChunkPos` public static `ZERO`, `INVALID_CHUNK_POS`, `REGION_SIZE` ve `REGION_MAX_INDEX` sabitlerini associated constant olarak karşılar. Rust adapter sabitleri Java ile aynı JSON şekline taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos public sabitleri için 4 özgün senaryo hazırlandı.
- `ZERO`, `INVALID_CHUNK_POS`, `REGION_SIZE` ve `REGION_MAX_INDEX` değerleri doğrudan Java/Rust API'lerinden karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1478/1478** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1980/1980** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1980 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `abe3d8cdf014db1ee9b2fa953ff1b7ba7bc40e31c9d35672c0cd8adb20896f50`. Birleşik referans kimliği: `e068e38cc1d0ab64ac2844f20d474f313e2325b42cc620af39051ca8aa6748cd`.

Adapter hash'leri: debug `97f6542ce3ae6bdc92e45b21488b106ff68a3970f00c179def9efd5e3e488ad7`, release `f551909bf9a55fa2fc2aae95704f10f5210192c5c00eaaba1c0afa39126f64609`.

Yerel raporlar: `compare-xrmyz06w` (math debug), `compare-zrxwzjp_` (math release), `compare-dxdmken0` (birleşik debug), `compare-cvif6gdr` (birleşik release) ve `verify-jlkp_8wf` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört public sabit sorgusu ve ilgili JSON çıktı şekilleriyle sınırlıdır. Private katsayılar, invalid sentinel'ın oyun akışındaki tüketicileri ve sabitlerin diğer istemci davranışlarındaki sonuçları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
