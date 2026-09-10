# P01.a — Mth createInsecureUUID doğrulaması

`src/math.rs` içindeki `mth_create_insecure_uuid`, Java `Mth.createInsecureUUID(RandomSource)` yolunu JSONL adapter'a bağlar. İki `nextLong` çekişi ve version/variant maskeleri Java oracle çıktısının `most_hex`/`least_hex` yarılarıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- On iki özgün signed i64 seed senaryosu hazırlandı. Sıfır, negatif ve yüksek bit seed'leri ile iki çekişin version 4 ve RFC 4122 variant maskeleri kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2023 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2023/2023 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2525/2525 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `1a217debf64171ea8f99e220ffd6aca1130ecceef3e6015b82f0c12a4e3f9f0d`. Birleşik referans kimliği: `38386c8d844bb2560f83f62571433316aca08b176233a98979cccf45ec22a8d5`.

Adapter hashleri: debug `0f53bfe9b4fa3f51b5f1cac8229fea64a4d485c5c5fae4f8b5607a300ecb94e8`, release `61ae7de80355a2cf73c393fbd30be18b82042d583a1918baf991fa8a22629e888`.

Yerel raporlar: `compare-4sx1lm2m` ve `compare-m3ib6sjn` (math debug/release), `compare-czvhou3w` ve `compare-oz19j96e` (birleşik debug/release), `verify-xjnq5poq` (math) ve `verify-nukwd4yc` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki `Mth.createInsecureUUID` yordamı ve bu dilimin 12 seed girdisiyle sınırlıdır. `Mth.getSeed`, RandomSource'un bağımsız dizisi ve UUID byte codec'i ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
