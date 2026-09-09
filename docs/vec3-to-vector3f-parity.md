# P01.b — Vec3 toVector3f doğrulaması

`src/math.rs`, Java `Vec3.toVector3f()` metodunu karşılar. Rust `Vec3f` değeri üç bileşeni Java'nın bağımsız f64 → f32 daraltmalarıyla üretir; Java oracle doğrudan `Vec3.toVector3f()` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1193 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1193/1193** math senaryosunda eşleşti.
- Altı yeni senaryoda temel f32 dönüşüm, f64 → f32 yuvarlama, signed zero, f64/f32 subnormal sınırları, sonlu değer taşması ve Infinity/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1695/1695** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1695 senaryoluk Java referansını yeniden doğruladı.

Math referansı `3a6a2f5a3d8ff6b65e565413ca95aeeb802f3ca965620ed874e898bcc52c5d61`, birleşik referans `5f58cd401556c40d3fc3eb7c3e511a8dbc2553c3390034c888e648d9d8a3ad4c` kimliğindedir. Math karşılaştırma raporları `compare-dgq7mp_7` (debug) ve `compare-2hmy86uj` (release); birleşik raporlar `compare-06i9lguy` (debug) ve `compare-r6guli2r` (release), Java doğrulaması `verify-k_vt31c0` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.toVector3f()` sonucunun üç f32 bileşen snapshot'ıyla sınırlıdır. `Vec3`'ün f64 işlemleri, `Vector3f` mutasyon yardımcıları ve diğer dönüşümler ayrı dilimlerdir.
