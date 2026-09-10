# P01.a — Mth frac yardımcıları doğrulaması

`src/math.rs` içindeki `frac_f32` ve `frac_f64` yordamları, Java `Mth.frac(float)` ve `Mth.frac(double)` yollarını JSONL adapter'a bağlar. Adapter beklenen değerleri üretmez; raw f32/f64 sonuç bitleri Java oracle çıktısıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- İki `Mth.frac` overload'u için 24 özgün senaryo hazırlandı. Pozitif/negatif kesir ve tam sayı yolları, signed zero, f32/f64 subnormal değerler, int/long üst sınır daraltması, Infinity ve NaN kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2124 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2124/2124 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2626/2626 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki `Mth.frac` yordamları ve bu dilimin 24 girdisiyle sınırlıdır. `floor`, `lfloor`, `ceil`, `ceilLong` ve `getSeed` ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
