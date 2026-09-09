# P01.b — Vec3 directionFromRotation doğrulaması

`src/math.rs`, Java `Vec3.directionFromRotation(float, float)` ve `directionFromRotation(Vec2)` overload'larını aynı f32 derece→radyan, Java `Mth` lookup ve f32 ürün sınırlarıyla karşılar. Java oracle, iki overload'u doğrudan çağırır; beklenen Vec3 snapshot'ı Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 859 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **859/859** math senaryosunda eşleşti.
- 18 yeni senaryoda iki overload, cardinal ve birleşik açıları, quantization, signed zero ve NaN/Infinity/taşma açılarını karşılaştırdı.
- Birleşik altı aile corpus'u **1313/1313** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1313 senaryoluk Java referansını yeniden doğruladı.

Math referansı `b65c658bf8c61413f639524850d054786ff936198bd6d0bb52662da232a61d77`, birleşik referans `5ddc1ee352aaf51a604b52f8e9f1d9356e75f176e3e2bdb0b5a00c406d6c51e2` kimliklerindedir. Math karşılaştırma raporları `compare-_2ksshx8` (debug) ve `compare-z3kwikqe` (release); birleşik raporlar `compare-6n7i4k79` (debug) ve `compare-se85bnok` (release), Java doğrulaması `verify-1_zhkgp3` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec3 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. [`Vec3.rotation()`](vec3-rotation-method-parity.md), local coordinate dönüşleri ve `Vec2.rotate` ayrı davranış dilimleridir.
