# P01.b — Vec3 toVector3f doğrulaması

`src/math.rs`, Java `Vec3.toVector3f()` metodunu karşılar. Rust `Vec3f` değeri üç bileşeni Java'nın bağımsız f64 → f32 daraltmalarıyla üretir; Java oracle doğrudan `Vec3.toVector3f()` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1019 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1019/1019** math senaryosunda eşleşti.
- Altı yeni senaryoda temel f32 dönüşüm, f64 → f32 yuvarlama, signed zero, f64/f32 subnormal sınırları, sonlu değer taşması ve Infinity/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1521/1521** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1521 senaryoluk Java referansını yeniden doğruladı.

Math referansı `5df2c7aa290a2d3deb2e318553a41a27dbe6b08b22e1d31b68362eedc36e2108`, birleşik referans `53cb6b5a4397567e26419e7466bcd8577c97308e626648d85f2a40bb55fb2406` kimliğindedir. Math karşılaştırma raporları `compare-_lwapdio` (debug) ve `compare-ed7p25g6` (release); birleşik raporlar `compare-uh_quimp` (debug) ve `compare-233_yr3a` (release), Java doğrulaması `verify-wqacm20n` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.toVector3f()` sonucunun üç f32 bileşen snapshot'ıyla sınırlıdır. `Vec3`'ün f64 işlemleri, `Vector3f` mutasyon yardımcıları ve diğer dönüşümler ayrı dilimlerdir.
