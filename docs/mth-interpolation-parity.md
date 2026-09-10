# P01.a — Mth lerp2/lerp3/catmullrom doğrulaması

`src/math.rs` içindeki `mth_lerp2_f64`, `mth_lerp3_f64` ve `mth_catmullrom_f32`, Java `Mth` interpolasyon yordamlarının iç içe çağrı ve kaynak sıralı polinom aritmetiğini taşır. `examples/behavior_adapter.rs` bu üretim yordamlarını JSONL senaryolarına bağlar; beklenen değerleri üretmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `lerp2`, `lerp3` ve `catmullrom` için her biri 20 olmak üzere 60 özgün senaryo hazırlandı. Alpha extrapolation'ı, ters ve signed-zero girdiler, subnormal değerler, hassasiyet sırası, taşma, Infinity ve NaN sonuçları kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 1843 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 1843/1843 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2345/2345 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `dfeb55b1b61cbc7649df7ae8af88469cbf7480a194892aacbcf8e5e5b93806bd`. Birleşik referans kimliği: `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576`.

Adapter hashleri: debug `c3f51b7830e3c0554138ff428a979a8f54ffaf85f7e2082bc6962bdcf01ff1d9`, release `fa350ee97911ed79b9b6db86a8af51e8b7dc87401e8312957b94741266b4b51b`.

Yerel raporlar: `compare-5_avciij` ve `compare-e6e6agr4` (math debug/release), `compare-gvo3eind` ve `compare-wmabbl5t` (birleşik debug/release), `verify-zbtpetmh` (math) ve `verify-vzenwlxd` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki üç `Mth` yordamı ve bu dilimin 60 girdisiyle sınırlıdır. Scalar `lerp`, `lerpInt`, `lerpDiscrete`, Vec3 `lerp` ve diğer Mth interpolasyon kullanıcıları ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
