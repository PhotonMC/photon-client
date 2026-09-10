# P01.a — Mth frac yardımcıları doğrulaması

`src/math.rs` içindeki `frac_f32` ve `frac_f64` yordamları, Java `Mth.frac(float)` ve `Mth.frac(double)` yollarını JSONL adapter'a bağlar. Adapter beklenen değerleri üretmez; raw f32/f64 sonuç bitleri Java oracle çıktısıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- İki `Mth.frac` overload'u için 24 özgün senaryo hazırlandı. Pozitif/negatif kesir ve tam sayı yolları, signed zero, f32/f64 subnormal değerler, int/long üst sınır daraltması, Infinity ve NaN kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2047 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2047/2047 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2549/2549 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `cf0843cad3317af0dc342572b3e0a3e0a9e482697ceeec73754750ceb27c15cd`. Birleşik referans kimliği: `510e1d4575f7dd92a09a6d3f896c2b8077a8fb2c01dd57ea32b9b48ce94a7583`.

Adapter hashleri: debug `794b1627f841615c5f200db08f1b8be0061ba9791a3dfa61ac439b9a2eace997`, release `986944806fde5347db60e40c7761984d390aaea2a0c4eceeeab651c2e0d30fd58`.

Yerel raporlar: `compare-j7dw7d6f` ve `compare-6na1_w6w` (math debug/release), `compare-qz7665of` ve `compare-2e2muf1s` (birleşik debug/release), `verify-e6j0lf9a` (math) ve `verify-s7mqu5wk` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki `Mth.frac` yordamları ve bu dilimin 24 girdisiyle sınırlıdır. `floor`, `lfloor`, `ceil`, `ceilLong` ve `getSeed` ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
