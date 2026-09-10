# P01.a — Mth güç-of-two yardımcıları doğrulaması

`src/math.rs` içindeki `smallest_encompassing_power_of_two`, `smallest_square_side`, `is_power_of_two_i32`, `is_power_of_two_i64`, `ceillog2` ve `log2` yordamları, Java `Mth` güç-of-two yardımcılarını JSONL adapter'a bağlar. Adapter beklenen değerleri üretmez; tamsayı sonuçları ve boolean sonuçları Java oracle çıktısıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Altı yordam için 10'ar olmak üzere 60 özgün senaryo hazırlandı. i32 taşması, negatif/yüksek bit girdileri, zero/non-power ayrımı, De Bruijn indekslemesi ve square-side sınırları kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2047 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2047/2047 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2549/2549 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `cf0843cad3317af0dc342572b3e0a3e0a9e482697ceeec73754750ceb27c15cd`. Birleşik referans kimliği: `510e1d4575f7dd92a09a6d3f896c2b8077a8fb2c01dd57ea32b9b48ce94a7583`.

Adapter hashleri: debug `794b1627f841615c5f200db08f1b8be0061ba9791a3dfa61ac439b9a2eace997`, release `986944806fde5347db60e40c7761984d390aaea2a0c4eceeeab651c2e0d30fd58`.

Yerel raporlar: `compare-j7dw7d6f` ve `compare-6na1_w6w` (math debug/release), `compare-qz7665of` ve `compare-2e2muf1s` (birleşik debug/release), `verify-e6j0lf9a` (math) ve `verify-s7mqu5wk` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki Mth güç-of-two yordamları ve bu dilimin 60 girdisiyle sınırlıdır. `frac`, `getSeed`, `atan2`, ters karekök, renk dönüşümü ve `binarySearch` ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
