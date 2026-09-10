# P01.b — Vec2 rotate(double) doğrulaması

`src/math.rs`, Java `Vec2.rotate(double)` davranışını f64 açı, Java `Mth` lookup sinüs/kosinüs değerleri ve f32 vektör işlemleriyle karşılar. Java oracle, `Vec2.rotate()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1987 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1987/1987** math senaryosunda eşleşti.
- 18 yeni senaryoda temel ve eksen dönüşleri, lookup quantization, signed zero, subnormal, sonlu sınır, Infinity, NaN ve f32 taşması karşılaştırıldı.
- Birleşik altı aile corpus'u **2489/2489** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2489 senaryoluk Java referansını yeniden doğruladı.

Math referansı `665bdda6827e71f559f4e9a959ed126f3d3e055594f3b677359c51d0d3a74e16`, birleşik referans `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc` kimliklerindedir. Math karşılaştırma raporları `compare-zt_h_egg` (debug) ve `compare-7afcvx21` (release); birleşik raporlar `compare-251aed0_` (debug) ve `compare-jqj2kta_` (release), Java doğrulaması `verify-csdh4b83` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. Vec2 add/dot/distance işlemleri, daha geniş input hareketi ve kamera dönüşümleri ayrı davranış dilimleridir.
