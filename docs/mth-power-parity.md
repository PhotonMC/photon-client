# P01.a — Mth güç-of-two yardımcıları doğrulaması

`src/math.rs` içindeki `smallest_encompassing_power_of_two`, `smallest_square_side`, `is_power_of_two_i32`, `is_power_of_two_i64`, `ceillog2` ve `log2` yordamları, Java `Mth` güç-of-two yardımcılarını JSONL adapter'a bağlar. Adapter beklenen değerleri üretmez; tamsayı sonuçları ve boolean sonuçları Java oracle çıktısıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Altı yordam için 10'ar olmak üzere 60 özgün senaryo hazırlandı. i32 taşması, negatif/yüksek bit girdileri, zero/non-power ayrımı, De Bruijn indekslemesi ve square-side sınırları kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 1999 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 1999/1999 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2501/2501 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `d0cf0bb203c8ea48ad8144fae6a58022846d1a5cfb95b398018620bae7b0bc3d`. Birleşik referans kimliği: `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040`.

Adapter hashleri: debug `5d019ee46fe3c7981d2881f496667dfc50b632b55a72f1c124178a69528f7705`, release `6081f381a8d8391d22750c81b039167dc2cb4dabf8274f6410be0463eb1881a8`.

Yerel raporlar: `compare-druvbg3h` ve `compare-k2ed8y70` (math debug/release), `compare-h1kbcaxz` ve `compare-js7_mas4` (birleşik debug/release), `verify-izjnelsu` (math) ve `verify-d2ptfzig` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki Mth güç-of-two yordamları ve bu dilimin 60 girdisiyle sınırlıdır. `frac`, `getSeed`, `atan2`, ters karekök, renk dönüşümü ve `binarySearch` ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
