# P01.b — Vec3 rotation() doğrulaması

`src/math.rs`, Java `Vec3.rotation()` davranışını f64 trigonometrik ara işlemleri, f32 açı dönüşümü ve Vec2 snapshot hesaplarıyla karşılar. Java oracle, `Vec3.rotation()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1031 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1031/1031** math senaryosunda eşleşti.
- 18 yeni senaryoda eksenler, birleşik ve non-unit vektörler, sıfır/signed-zero, subnormal, sonlu sınırlar ve Infinity/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1533/1533** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1533 senaryoluk Java referansını yeniden doğruladı.

Math referansı `cbae17cd88ce5b1617d01034d27ad104db9b471eefc8bebf83ed4e8d1dc0aa70`, birleşik referans `338731e44c45c8acc6fbd70e7fe1439cdaa28ec5996f6a4d9517412cc50106a0` kimliklerindedir. Math karşılaştırma raporları `compare-2ftkouxd` (debug) ve `compare-g204xxpd` (release); birleşik raporlar `compare-ag9x0lbh` (debug) ve `compare-sl2ed6ln` (release), Java doğrulaması `verify-5bpx38hs` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. `Vec3.directionFromRotation`, `Vec2.rotate`, kamera yönlendirmesi ve local coordinate dönüşleri ayrı davranış dilimleridir.
