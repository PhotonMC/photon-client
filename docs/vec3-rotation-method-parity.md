# P01.b — Vec3 rotation() doğrulaması

`src/math.rs`, Java `Vec3.rotation()` davranışını f64 trigonometrik ara işlemleri, f32 açı dönüşümü ve Vec2 snapshot hesaplarıyla karşılar. Java oracle, `Vec3.rotation()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1354 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1354/1354** math senaryosunda eşleşti.
- 18 yeni senaryoda eksenler, birleşik ve non-unit vektörler, sıfır/signed-zero, subnormal, sonlu sınırlar ve Infinity/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1856/1856** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1856 senaryoluk Java referansını yeniden doğruladı.

Math referansı `7bacfa4674e388b5f38f1b0a041e5531afe27e845d4374895c79e4fb56697a24`, birleşik referans `3afb23055305f62e13f7bdef0f8d340c8294187f00de3b26a2c52874e45642a7` kimliklerindedir. Math karşılaştırma raporları `compare-z9ypl4_s` (debug) ve `compare-0xhgd0yz` (release); birleşik raporlar `compare-ote21q3t` (debug) ve `compare-r5nijsku` (release), Java doğrulaması `verify-2b2qykhl` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. `Vec3.directionFromRotation`, `Vec2.rotate`, kamera yönlendirmesi ve local coordinate dönüşleri ayrı davranış dilimleridir.
