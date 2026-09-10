# P01.a — Mth frac yardımcıları doğrulaması

`src/math.rs` içindeki `frac_f32` ve `frac_f64` yordamları, Java `Mth.frac(float)` ve `Mth.frac(double)` yollarını JSONL adapter'a bağlar. Adapter beklenen değerleri üretmez; raw f32/f64 sonuç bitleri Java oracle çıktısıyla karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- İki `Mth.frac` overload'u için 24 özgün senaryo hazırlandı. Pozitif/negatif kesir ve tam sayı yolları, signed zero, f32/f64 subnormal değerler, int/long üst sınır daraltması, Infinity ve NaN kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 1987 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 1987/1987 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2489/2489 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `665bdda6827e71f559f4e9a959ed126f3d3e055594f3b677359c51d0d3a74e16`. Birleşik referans kimliği: `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc`.

Adapter hashleri: debug `bf472fbe984c9033c542831c832c85f3ea36283e022040542677049db4615a06`, release `04de447e6d530b9e66134316577c0e326f349761b64ee41c18f4e76fd6d3b226`.

Yerel raporlar: `compare-zt_h_egg` ve `compare-7afcvx21` (math debug/release), `compare-251aed0_` ve `compare-jqj2kta_` (birleşik debug/release), `verify-5v_ilol4` (math) ve `verify-csdh4b83` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki `Mth.frac` yordamları ve bu dilimin 24 girdisiyle sınırlıdır. `floor`, `lfloor`, `ceil`, `ceilLong` ve `getSeed` ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
