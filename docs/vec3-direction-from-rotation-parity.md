# P01.b — Vec3 directionFromRotation doğrulaması

`src/math.rs`, Java `Vec3.directionFromRotation(float, float)` ve `directionFromRotation(Vec2)` overload'larını aynı f32 derece→radyan, Java `Mth` lookup ve f32 ürün sınırlarıyla karşılar. Java oracle, iki overload'u doğrudan çağırır; beklenen Vec3 snapshot'ı Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 953 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **953/953** math senaryosunda eşleşti.
- 18 yeni senaryoda iki overload, cardinal ve birleşik açıları, quantization, signed zero ve NaN/Infinity/taşma açılarını karşılaştırdı.
- Birleşik altı aile corpus'u **1455/1455** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1455 senaryoluk Java referansını yeniden doğruladı.

Math referansı `c05b4f6d5d95b034d9034387072c87ac36f1b31427b1fbbbbebe885dec1bfc3b`, birleşik referans `e267ec135794f878c9e54d22faa3451ffe125c25ad17b7ad67e223c1d1fb3b55` kimliklerindedir. Math karşılaştırma raporları `compare-j1r42nvx` (debug) ve `compare-yls3sbu1` (release); birleşik raporlar `compare-ttkrbj5a` (debug) ve `compare-dz8xzceq` (release), Java doğrulaması `verify-zgd9uosd` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec3 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. [`Vec3.rotation()`](vec3-rotation-method-parity.md), local coordinate dönüşleri ve `Vec2.rotate` ayrı davranış dilimleridir.
