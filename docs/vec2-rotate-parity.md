# P01.b — Vec2 rotate(double) doğrulaması

`src/math.rs`, Java `Vec2.rotate(double)` davranışını f64 açı, Java `Mth` lookup sinüs/kosinüs değerleri ve f32 vektör işlemleriyle karşılar. Java oracle, `Vec2.rotate()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1450 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1450/1450** math senaryosunda eşleşti.
- 18 yeni senaryoda temel ve eksen dönüşleri, lookup quantization, signed zero, subnormal, sonlu sınır, Infinity, NaN ve f32 taşması karşılaştırıldı.
- Birleşik altı aile corpus'u **1952/1952** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1952 senaryoluk Java referansını yeniden doğruladı.

Math referansı `fb79a016fcf5041a8631f665991eaad422d1346ef0c1c0c12f2166cdb0057846`, birleşik referans `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11` kimliklerindedir. Math karşılaştırma raporları `compare-qtqxwt9b` (debug) ve `compare-ebava42g` (release); birleşik raporlar `compare-x0hkkvkv` (debug) ve `compare-x5wgwswa` (release), Java doğrulaması `verify-rc21vyy9` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. Vec2 add/dot/distance işlemleri, daha geniş input hareketi ve kamera dönüşümleri ayrı davranış dilimleridir.
