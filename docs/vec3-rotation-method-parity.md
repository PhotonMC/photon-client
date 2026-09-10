# P01.b — Vec3 rotation() doğrulaması

`src/math.rs`, Java `Vec3.rotation()` davranışını f64 trigonometrik ara işlemleri, f32 açı dönüşümü ve Vec2 snapshot hesaplarıyla karşılar. Java oracle, `Vec3.rotation()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1963 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1963/1963** math senaryosunda eşleşti.
- 18 yeni senaryoda eksenler, birleşik ve non-unit vektörler, sıfır/signed-zero, subnormal, sonlu sınırlar ve Infinity/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2465/2465** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2465 senaryoluk Java referansını yeniden doğruladı.

Math referansı `461f1ba035270ce58c69f06fec7167bb0517cd22aed6c78c25e0b076684b28fa`, birleşik referans `ef5a23b696bb32be24dcf9aa656adce55733813eb85a06cfd10f6418b5a4c573` kimliklerindedir. Math karşılaştırma raporları `compare-hiau72j0` (debug) ve `compare-bttpme0o` (release); birleşik raporlar `compare-k17hah5m` (debug) ve `compare-hiiz6u35` (release), Java doğrulaması `verify-82qi_x56` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. `Vec3.directionFromRotation`, `Vec2.rotate`, kamera yönlendirmesi ve local coordinate dönüşleri ayrı davranış dilimleridir.
