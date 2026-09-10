# P01.a — Mth createInsecureUUID doğrulaması

`src/math.rs` içindeki `mth_create_insecure_uuid`, Java `Mth.createInsecureUUID(RandomSource)` yolunu JSONL adapter'a bağlar. İki `nextLong` çekişi ve version/variant maskeleri Java oracle çıktısının `most_hex`/`least_hex` yarılarıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- On iki özgün signed i64 seed senaryosu hazırlandı. Sıfır, negatif ve yüksek bit seed'leri ile iki çekişin version 4 ve RFC 4122 variant maskeleri kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 1999 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 1999/1999 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2501/2501 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `d0cf0bb203c8ea48ad8144fae6a58022846d1a5cfb95b398018620bae7b0bc3d`. Birleşik referans kimliği: `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040`.

Adapter hashleri: debug `5d019ee46fe3c7981d2881f496667dfc50b632b55a72f1c124178a69528f7705`, release `6081f381a8d8391d22750c81b039167dc2cb4dabf8274f6410be0463eb1881a8`.

Yerel raporlar: `compare-druvbg3h` ve `compare-k2ed8y70` (math debug/release), `compare-h1kbcaxz` ve `compare-js7_mas4` (birleşik debug/release), `verify-izjnelsu` (math) ve `verify-d2ptfzig` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki `Mth.createInsecureUUID` yordamı ve bu dilimin 12 seed girdisiyle sınırlıdır. `Mth.getSeed`, RandomSource'un bağımsız dizisi ve UUID byte codec'i ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
