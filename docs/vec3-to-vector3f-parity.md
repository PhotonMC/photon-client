# P01.b — Vec3 toVector3f doğrulaması

`src/math.rs`, Java `Vec3.toVector3f()` metodunu karşılar. Rust `Vec3f` değeri üç bileşeni Java'nın bağımsız f64 → f32 daraltmalarıyla üretir; Java oracle doğrudan `Vec3.toVector3f()` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 875 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **875/875** math senaryosunda eşleşti.
- Altı yeni senaryoda temel f32 dönüşüm, f64 → f32 yuvarlama, signed zero, f64/f32 subnormal sınırları, sonlu değer taşması ve Infinity/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1329/1329** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1329 senaryoluk Java referansını yeniden doğruladı.

Math referansı `5613b7aa336a4fb4cf729c33b1e4d9566e5c0795c663ab83879f3787157f0831`, birleşik referans `22a5e87b3b46a0afade4d43a8a9e0dfaf96d8d892ea18ddc0ba5ebf77750772c` kimliğindedir. Math karşılaştırma raporları `compare-p4m_t_r8` (debug) ve `compare-eskrxypz` (release); birleşik raporlar `compare-skwbedqw` (debug) ve `compare-9bwuf8cs` (release), Java doğrulaması `verify-5pjzzzfd` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.toVector3f()` sonucunun üç f32 bileşen snapshot'ıyla sınırlıdır. `Vec3`'ün f64 işlemleri, `Vector3f` mutasyon yardımcıları ve diğer dönüşümler ayrı dilimlerdir.
