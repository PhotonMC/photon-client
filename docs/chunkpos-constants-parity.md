# P01.c — ChunkPos public sabitleri doğrulaması

`src/binary.rs`, Java `ChunkPos` public static `ZERO`, `INVALID_CHUNK_POS`, `REGION_SIZE` ve `REGION_MAX_INDEX` sabitlerini associated constant olarak karşılar. Rust adapter sabitleri Java ile aynı JSON şekline taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos public sabitleri için 4 özgün senaryo hazırlandı.
- `ZERO`, `INVALID_CHUNK_POS`, `REGION_SIZE` ve `REGION_MAX_INDEX` değerleri doğrudan Java/Rust API'lerinden karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2047/2047** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2549/2549** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2549 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `cf0843cad3317af0dc342572b3e0a3e0a9e482697ceeec73754750ceb27c15cd`. Birleşik referans kimliği: `510e1d4575f7dd92a09a6d3f896c2b8077a8fb2c01dd57ea32b9b48ce94a7583`.

Adapter hash'leri: debug `97f6542ce3ae6bdc92e45b21488b106ff68a3970f00c179def9efd5e3e488ad7`, release `f551909bf9a55fa2fc2aae95704f10f5210192c5c00eaaba1c0afa39126f64609`.

Yerel raporlar: `compare-j7dw7d6f` (math debug), `compare-6na1_w6w` (math release), `compare-qz7665of` (birleşik debug), `compare-2e2muf1s` (birleşik release) ve `verify-s7mqu5wk` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört public sabit sorgusu ve ilgili JSON çıktı şekilleriyle sınırlıdır. Private katsayılar, invalid sentinel'ın oyun akışındaki tüketicileri ve sabitlerin diğer istemci davranışlarındaki sonuçları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
