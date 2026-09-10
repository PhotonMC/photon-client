# P01.b — Vec3 directionFromRotation doğrulaması

`src/math.rs`, Java `Vec3.directionFromRotation(float, float)` ve `directionFromRotation(Vec2)` overload'larını aynı f32 derece→radyan, Java `Mth` lookup ve f32 ürün sınırlarıyla karşılar. Java oracle, iki overload'u doğrudan çağırır; beklenen Vec3 snapshot'ı Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2076 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2076/2076** math senaryosunda eşleşti.
- 18 yeni senaryoda iki overload, cardinal ve birleşik açıları, quantization, signed zero ve NaN/Infinity/taşma açılarını karşılaştırdı.
- Birleşik altı aile corpus'u **2578/2578** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2578 senaryoluk Java referansını yeniden doğruladı.

Math referansı `e84d285a8cd335bd1ba5cd88d076e7cb38315013b991057804c6f1b4598f1c80`, birleşik referans `9c94496c39504f2bd146d331630516f020e05259c67c0bca659d1f1ca92c74a8` kimliklerindedir. Math karşılaştırma raporları `compare-k2v65l83` (debug) ve `compare-9uf1gox2` (release); birleşik raporlar `compare-ptlm9ptr` (debug) ve `compare-86z18x_k` (release), Java doğrulaması `verify-31k8bjcu` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec3 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. [`Vec3.rotation()`](vec3-rotation-method-parity.md), local coordinate dönüşleri ve `Vec2.rotate` ayrı davranış dilimleridir.
