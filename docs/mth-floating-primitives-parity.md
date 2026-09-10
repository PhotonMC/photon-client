# P01.a — Mth kayan nokta temel yardımcıları doğrulaması

`src/math.rs` içindeki `mth_sin_f64`, `mth_cos_f64`, `mth_sqrt_f32`, `mth_abs_f32` ve mevcut hassasiyetli clamp yolları, Java `Mth` kayan nokta temel yordamlarını JSONL adapter'a bağlar. Adapter beklenen değerleri üretmez; her sonuç Java oracle çıktısındaki raw IEEE-754 bitleriyle karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `sin` ve `cos` için 10'ar, `sqrt`, `abs` ve f32/f64 `clamp` için 10'ar olmak üzere 60 özgün senaryo hazırlandı. Lookup kuantizasyonu, f32/f64 hassasiyeti, signed zero, subnormal, Infinity, NaN payload'ı ve ters clamp sınırları kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 2076 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 2076/2076 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2578/2578 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `e84d285a8cd335bd1ba5cd88d076e7cb38315013b991057804c6f1b4598f1c80`. Birleşik referans kimliği: `9c94496c39504f2bd146d331630516f020e05259c67c0bca659d1f1ca92c74a8`.

Adapter hashleri: debug `6eb1e772a2e06c20b8d0c09ba0b2a11b5f1df7a671ee8d1513127bc318d143ef`, release `318c5ef22a8d56277aa19f9da9f5becca9f575904eba79756b959bdc8c993fe88`.

Yerel raporlar: `compare-k2v65l83` ve `compare-9uf1gox2` (math debug/release), `compare-ptlm9ptr` ve `compare-86z18x_k` (birleşik debug/release), `verify-9dtf6rea` (math) ve `verify-31k8bjcu` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki Mth kayan nokta temel yordamları ve bu dilimin 60 girdisiyle sınırlıdır. `floor`, `ceil`, `lfloor`, `clampedLerp` ve diğer Mth kullanıcıları ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
