# P01.b — Vec3 toVector3f doğrulaması

`src/math.rs`, Java `Vec3.toVector3f()` metodunu karşılar. Rust `Vec3f` değeri üç bileşeni Java'nın bağımsız f64 → f32 daraltmalarıyla üretir; Java oracle doğrudan `Vec3.toVector3f()` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 813 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **813/813** math senaryosunda eşleşti.
- Altı yeni senaryoda temel f32 dönüşüm, f64 → f32 yuvarlama, signed zero, f64/f32 subnormal sınırları, sonlu değer taşması ve Infinity/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1267/1267** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1267 senaryoluk Java referansını yeniden doğruladı.

Math referansı `28f7cb5106f4afaea9ccbb7f1a5601e99e2bf46d20a240f2d4288eb8fa0810ca`, birleşik referans `6b1e215c94a31ad5e80b7ab921054436873666bf0c583fb7b768cc855632bde2` kimliğindedir. Math karşılaştırma raporları `compare-nv2dy1tn` (debug) ve `compare-zi0twrtb` (release); birleşik raporlar `compare-lny6p0bp` (debug) ve `compare-g4_lvtxx` (release), Java doğrulaması `verify-nf8ea9dc` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.toVector3f()` sonucunun üç f32 bileşen snapshot'ıyla sınırlıdır. `Vec3`'ün f64 işlemleri, `Vector3f` mutasyon yardımcıları ve diğer dönüşümler ayrı dilimlerdir.
