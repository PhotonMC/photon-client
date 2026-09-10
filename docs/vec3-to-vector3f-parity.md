# P01.b — Vec3 toVector3f doğrulaması

`src/math.rs`, Java `Vec3.toVector3f()` metodunu karşılar. Rust `Vec3f` değeri üç bileşeni Java'nın bağımsız f64 → f32 daraltmalarıyla üretir; Java oracle doğrudan `Vec3.toVector3f()` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1450 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1450/1450** math senaryosunda eşleşti.
- Altı yeni senaryoda temel f32 dönüşüm, f64 → f32 yuvarlama, signed zero, f64/f32 subnormal sınırları, sonlu değer taşması ve Infinity/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1952/1952** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1952 senaryoluk Java referansını yeniden doğruladı.

Math referansı `fb79a016fcf5041a8631f665991eaad422d1346ef0c1c0c12f2166cdb0057846`, birleşik referans `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11` kimliğindedir. Math karşılaştırma raporları `compare-qtqxwt9b` (debug) ve `compare-ebava42g` (release); birleşik raporlar `compare-x0hkkvkv` (debug) ve `compare-x5wgwswa` (release), Java doğrulaması `verify-rc21vyy9` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.toVector3f()` sonucunun üç f32 bileşen snapshot'ıyla sınırlıdır. `Vec3`'ün f64 işlemleri, `Vector3f` mutasyon yardımcıları ve diğer dönüşümler ayrı dilimlerdir.
