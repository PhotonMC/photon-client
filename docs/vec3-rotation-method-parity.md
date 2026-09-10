# P01.b — Vec3 rotation() doğrulaması

`src/math.rs`, Java `Vec3.rotation()` davranışını f64 trigonometrik ara işlemleri, f32 açı dönüşümü ve Vec2 snapshot hesaplarıyla karşılar. Java oracle, `Vec3.rotation()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2023 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2023/2023** math senaryosunda eşleşti.
- 18 yeni senaryoda eksenler, birleşik ve non-unit vektörler, sıfır/signed-zero, subnormal, sonlu sınırlar ve Infinity/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2525/2525** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2525 senaryoluk Java referansını yeniden doğruladı.

Math referansı `1a217debf64171ea8f99e220ffd6aca1130ecceef3e6015b82f0c12a4e3f9f0d`, birleşik referans `38386c8d844bb2560f83f62571433316aca08b176233a98979cccf45ec22a8d5` kimliklerindedir. Math karşılaştırma raporları `compare-4sx1lm2m` (debug) ve `compare-m3ib6sjn` (release); birleşik raporlar `compare-czvhou3w` (debug) ve `compare-oz19j96e` (release), Java doğrulaması `verify-nukwd4yc` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. `Vec3.directionFromRotation`, `Vec2.rotate`, kamera yönlendirmesi ve local coordinate dönüşleri ayrı davranış dilimleridir.
