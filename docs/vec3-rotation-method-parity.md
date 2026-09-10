# P01.b — Vec3 rotation() doğrulaması

`src/math.rs`, Java `Vec3.rotation()` davranışını f64 trigonometrik ara işlemleri, f32 açı dönüşümü ve Vec2 snapshot hesaplarıyla karşılar. Java oracle, `Vec3.rotation()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1378 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1378/1378** math senaryosunda eşleşti.
- 18 yeni senaryoda eksenler, birleşik ve non-unit vektörler, sıfır/signed-zero, subnormal, sonlu sınırlar ve Infinity/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1880/1880** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1880 senaryoluk Java referansını yeniden doğruladı.

Math referansı `6d35da84e1eb81c28088bf3f4951651b252daed29eda65c96f035389b897e6e4`, birleşik referans `d1375ef4dcd71b8eb3c95f84ca200ef0ed3303aa7500447f6683cb24a4bd1d81` kimliklerindedir. Math karşılaştırma raporları `compare-_ohp5wb8` (debug) ve `compare-bezugwpw` (release); birleşik raporlar `compare-i4o7vofo` (debug) ve `compare-9vhvfokc` (release), Java doğrulaması `verify-kfxghdw3` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. `Vec3.directionFromRotation`, `Vec2.rotate`, kamera yönlendirmesi ve local coordinate dönüşleri ayrı davranış dilimleridir.
