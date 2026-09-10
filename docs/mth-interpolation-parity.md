# P01.a — Mth lerp2/lerp3/catmullrom doğrulaması

`src/math.rs` içindeki `mth_lerp2_f64`, `mth_lerp3_f64` ve `mth_catmullrom_f32`, Java `Mth` interpolasyon yordamlarının iç içe çağrı ve kaynak sıralı polinom aritmetiğini taşır. `examples/behavior_adapter.rs` bu üretim yordamlarını JSONL senaryolarına bağlar; beklenen değerleri üretmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `lerp2`, `lerp3` ve `catmullrom` için her biri 20 olmak üzere 60 özgün senaryo hazırlandı. Alpha extrapolation'ı, ters ve signed-zero girdiler, subnormal değerler, hassasiyet sırası, taşma, Infinity ve NaN sonuçları kapsandı.
- Math referansı iki ayrı JVM çalıştırmasıyla 1987 senaryoda kararlı bulundu.
- Rust debug ve release adaptörleri güncel 1987/1987 math senaryosunda eşleşti.
- Birleşik altı aile corpus'u 2489/2489 senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `665bdda6827e71f559f4e9a959ed126f3d3e055594f3b677359c51d0d3a74e16`. Birleşik referans kimliği: `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc`.

Adapter hashleri: debug `bf472fbe984c9033c542831c832c85f3ea36283e022040542677049db4615a06`, release `04de447e6d530b9e66134316577c0e326f349761b64ee41c18f4e76fd6d3b226`.

Yerel raporlar: `compare-zt_h_egg` ve `compare-7afcvx21` (math debug/release), `compare-251aed0_` ve `compare-jqj2kta_` (birleşik debug/release), `verify-5v_ilol4` (math) ve `verify-csdh4b83` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansındaki üç `Mth` yordamı ve bu dilimin 60 girdisiyle sınırlıdır. Scalar `lerp`, `lerpInt`, `lerpDiscrete`, Vec3 `lerp` ve diğer Mth interpolasyon kullanıcıları ayrı kapsamdır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
