# P01.b — Vec3 directionFromRotation doğrulaması

`src/math.rs`, Java `Vec3.directionFromRotation(float, float)` ve `directionFromRotation(Vec2)` overload'larını aynı f32 derece→radyan, Java `Mth` lookup ve f32 ürün sınırlarıyla karşılar. Java oracle, iki overload'u doğrudan çağırır; beklenen Vec3 snapshot'ı Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1019 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1019/1019** math senaryosunda eşleşti.
- 18 yeni senaryoda iki overload, cardinal ve birleşik açıları, quantization, signed zero ve NaN/Infinity/taşma açılarını karşılaştırdı.
- Birleşik altı aile corpus'u **1521/1521** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1521 senaryoluk Java referansını yeniden doğruladı.

Math referansı `5df2c7aa290a2d3deb2e318553a41a27dbe6b08b22e1d31b68362eedc36e2108`, birleşik referans `53cb6b5a4397567e26419e7466bcd8577c97308e626648d85f2a40bb55fb2406` kimliklerindedir. Math karşılaştırma raporları `compare-_lwapdio` (debug) ve `compare-ed7p25g6` (release); birleşik raporlar `compare-uh_quimp` (debug) ve `compare-233_yr3a` (release), Java doğrulaması `verify-wqacm20n` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec3 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. [`Vec3.rotation()`](vec3-rotation-method-parity.md), local coordinate dönüşleri ve `Vec2.rotate` ayrı davranış dilimleridir.
