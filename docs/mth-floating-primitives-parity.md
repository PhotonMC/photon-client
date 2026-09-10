# P01.a — Mth kayan nokta temel yardımcıları doğrulaması

`src/math.rs` içindeki `mth_sin_f64`, `mth_cos_f64`, `mth_sqrt_f32`, `mth_abs_f32` ve mevcut hassasiyetli clamp yolları, Java `Mth` kayan nokta temel yordamlarını JSONL adapter'a bağlar. Adapter beklenen değerleri üretmez; her sonuç Java oracle çıktısındaki raw IEEE-754 bitleriyle karşılaştırılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `sin` ve `cos` için 10'ar, `sqrt`, `abs` ve f32/f64 `clamp` için 10'ar olmak üzere 60 özgün senaryo hazırlandı. Lookup kuantizasyonu, f32/f64 hassasiyeti, signed zero, subnormal, Infinity, NaN payload'ı ve ters clamp sınırları kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 1903 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 1903/1903 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2405/2405 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `0cc821a10c9af88c8563c153e55d57b691a8c69e33d90a8bd3022a5b3410bfd0`. Birleşik referans kimliği: `2daaa6bb28eca57adfb18563e32f011d764209b98ec6de5934a92a72e80abdfa`.

Adapter hashleri: debug `b2f092b374faa99b0fbb325d175fd2ca57c0c81b8a1b913240e9305da476fe74`, release `c1bd320746e6a2b869befca2f1abe4e529fccd8e94fdd8204a5cd0ca6176269a`.

Yerel raporlar: `compare-xiw125c3` ve `compare-v5h63z7q` (math debug/release), `compare-chdvlz2_` ve `compare-oywxz0ky` (birleşik debug/release), `verify-f023qdmh` (math) ve `verify-da0drjib` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki Mth kayan nokta temel yordamları ve bu dilimin 60 girdisiyle sınırlıdır. `floor`, `ceil`, `lfloor`, `clampedLerp` ve diğer Mth kullanıcıları ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
