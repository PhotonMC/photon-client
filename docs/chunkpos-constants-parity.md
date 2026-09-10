# P01.c — ChunkPos public sabitleri doğrulaması

`src/binary.rs`, Java `ChunkPos` public static `ZERO`, `INVALID_CHUNK_POS`, `REGION_SIZE` ve `REGION_MAX_INDEX` sabitlerini associated constant olarak karşılar. Rust adapter sabitleri Java ile aynı JSON şekline taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos public sabitleri için 4 özgün senaryo hazırlandı.
- `ZERO`, `INVALID_CHUNK_POS`, `REGION_SIZE` ve `REGION_MAX_INDEX` değerleri doğrudan Java/Rust API'lerinden karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1593/1593** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2095/2095** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2095 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `ec7f2431c2fc0a873c374068f226b4309821be58380c7e5eadccdc15fe690723`. Birleşik referans kimliği: `17d40db8607a5df22fc08a42ccd0065b6ffe818cef98507474d8663297c0ac04`.

Adapter hash'leri: debug `97f6542ce3ae6bdc92e45b21488b106ff68a3970f00c179def9efd5e3e488ad7`, release `f551909bf9a55fa2fc2aae95704f10f5210192c5c00eaaba1c0afa39126f64609`.

Yerel raporlar: `compare-gqgspijr` (math debug), `compare-z8zpkwnx` (math release), `compare-94x3ynna` (birleşik debug), `compare-2cii6gte` (birleşik release) ve `verify-lu6dvbz4` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört public sabit sorgusu ve ilgili JSON çıktı şekilleriyle sınırlıdır. Private katsayılar, invalid sentinel'ın oyun akışındaki tüketicileri ve sabitlerin diğer istemci davranışlarındaki sonuçları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
