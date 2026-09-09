# P01.b — Vec3 directionFromRotation doğrulaması

`src/math.rs`, Java `Vec3.directionFromRotation(float, float)` ve `directionFromRotation(Vec2)` overload'larını aynı f32 derece→radyan, Java `Mth` lookup ve f32 ürün sınırlarıyla karşılar. Java oracle, iki overload'u doğrudan çağırır; beklenen Vec3 snapshot'ı Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 989 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **989/989** math senaryosunda eşleşti.
- 18 yeni senaryoda iki overload, cardinal ve birleşik açıları, quantization, signed zero ve NaN/Infinity/taşma açılarını karşılaştırdı.
- Birleşik altı aile corpus'u **1491/1491** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1491 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9df41e71098bfa6c6de40d3789b0153838da1ca1634d3f012c1af262d09ffd35`, birleşik referans `0f630454331682d5ce34027e78b67ab2555fdb80eaa4f4e4bb2485885abebab6` kimliklerindedir. Math karşılaştırma raporları `compare-jccgb79g` (debug) ve `compare-r5e2j3zv` (release); birleşik raporlar `compare-n67o4lpe` (debug) ve `compare-y4v3jm1g` (release), Java doğrulaması `verify-hq80hpap` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec3 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. [`Vec3.rotation()`](vec3-rotation-method-parity.md), local coordinate dönüşleri ve `Vec2.rotate` ayrı davranış dilimleridir.
