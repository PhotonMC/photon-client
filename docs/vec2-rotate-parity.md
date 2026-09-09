# P01.b — Vec2 rotate(double) doğrulaması

`src/math.rs`, Java `Vec2.rotate(double)` davranışını f64 açı, Java `Mth` lookup sinüs/kosinüs değerleri ve f32 vektör işlemleriyle karşılar. Java oracle, `Vec2.rotate()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 849 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **849/849** math senaryosunda eşleşti.
- 18 yeni senaryoda temel ve eksen dönüşleri, lookup quantization, signed zero, subnormal, sonlu sınır, Infinity, NaN ve f32 taşması karşılaştırıldı.
- Birleşik altı aile corpus'u **1303/1303** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1303 senaryoluk Java referansını yeniden doğruladı.

Math referansı `e29959e1e804ec49cc4359adf306d0af3b2268e8c97cc1a9a11aeb3872e657a2`, birleşik referans `2934949cd24cd9684ec3034e431974ec99773d6473c977dd6771ec28e892c647` kimliklerindedir. Math karşılaştırma raporları `compare-huwe3iyu` (debug) ve `compare-mentlii0` (release); birleşik raporlar `compare-fw1d9dde` (debug) ve `compare-lx5vlkq_` (release), Java doğrulaması `verify-mw2w2ovh` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. Vec2 add/dot/distance işlemleri, daha geniş input hareketi ve kamera dönüşümleri ayrı davranış dilimleridir.
