# P01.a — Mth frac yardımcıları doğrulaması

`src/math.rs` içindeki `frac_f32` ve `frac_f64` yordamları, Java `Mth.frac(float)` ve `Mth.frac(double)` yollarını JSONL adapter'a bağlar. Adapter beklenen değerleri üretmez; raw f32/f64 sonuç bitleri Java oracle çıktısıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- İki `Mth.frac` overload'u için 24 özgün senaryo hazırlandı. Pozitif/negatif kesir ve tam sayı yolları, signed zero, f32/f64 subnormal değerler, int/long üst sınır daraltması, Infinity ve NaN kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2023 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2023/2023 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2525/2525 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `1a217debf64171ea8f99e220ffd6aca1130ecceef3e6015b82f0c12a4e3f9f0d`. Birleşik referans kimliği: `38386c8d844bb2560f83f62571433316aca08b176233a98979cccf45ec22a8d5`.

Adapter hashleri: debug `0f53bfe9b4fa3f51b5f1cac8229fea64a4d485c5c5fae4f8b5607a300ecb94e8`, release `61ae7de80355a2cf73c393fbd30be18b82042d583a1918baf991fa8a22629e888`.

Yerel raporlar: `compare-4sx1lm2m` ve `compare-m3ib6sjn` (math debug/release), `compare-czvhou3w` ve `compare-oz19j96e` (birleşik debug/release), `verify-xjnq5poq` (math) ve `verify-nukwd4yc` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki `Mth.frac` yordamları ve bu dilimin 24 girdisiyle sınırlıdır. `floor`, `lfloor`, `ceil`, `ceilLong` ve `getSeed` ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
