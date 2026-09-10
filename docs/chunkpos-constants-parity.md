# P01.c — ChunkPos public sabitleri doğrulaması

`src/binary.rs`, Java `ChunkPos` public static `ZERO`, `INVALID_CHUNK_POS`, `REGION_SIZE` ve `REGION_MAX_INDEX` sabitlerini associated constant olarak karşılar. Rust adapter sabitleri Java ile aynı JSON şekline taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos public sabitleri için 4 özgün senaryo hazırlandı.
- `ZERO`, `INVALID_CHUNK_POS`, `REGION_SIZE` ve `REGION_MAX_INDEX` değerleri doğrudan Java/Rust API'lerinden karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2100/2100** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2602/2602** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2602 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `f84b8d0a8b920d9e5455aefeebd2a9d768b7c9af5ace5dd1c90ff816a9e20abb`. Birleşik referans kimliği: `ec46e2f5bb40c4fdc045039ffcf03e500f593e7a5659dcd76ba9519601df03f2`.

Adapter hash'leri: debug `97f6542ce3ae6bdc92e45b21488b106ff68a3970f00c179def9efd5e3e488ad7`, release `f551909bf9a55fa2fc2aae95704f10f5210192c5c00eaaba1c0afa39126f64609`.

Yerel raporlar: `compare-f8rnhap6` (math debug), `compare-9ykm10o3` (math release), `compare-qo27kk4p` (birleşik debug), `compare-066pmptb` (birleşik release) ve `verify-icag4r7w` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört public sabit sorgusu ve ilgili JSON çıktı şekilleriyle sınırlıdır. Private katsayılar, invalid sentinel'ın oyun akışındaki tüketicileri ve sabitlerin diğer istemci davranışlarındaki sonuçları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
