# P01.b — Vec2 rotate(double) doğrulaması

`src/math.rs`, Java `Vec2.rotate(double)` davranışını f64 açı, Java `Mth` lookup sinüs/kosinüs değerleri ve f32 vektör işlemleriyle karşılar. Java oracle, `Vec2.rotate()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 947 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **947/947** math senaryosunda eşleşti.
- 18 yeni senaryoda temel ve eksen dönüşleri, lookup quantization, signed zero, subnormal, sonlu sınır, Infinity, NaN ve f32 taşması karşılaştırıldı.
- Birleşik altı aile corpus'u **1449/1449** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1449 senaryoluk Java referansını yeniden doğruladı.

Math referansı `173698891dc93629abbcd5ca3642ca5af04a5ddb10ffcbaa0610cf03306c320e`, birleşik referans `304e1c88da43eaa3ceb24663a0ab71ddb6fb1f5487c293a9f94fb7224b94c363` kimliklerindedir. Math karşılaştırma raporları `compare-auuk33g2` (debug) ve `compare-w_2b2455` (release); birleşik raporlar `compare-tjsmsqxw` (debug) ve `compare-sks6l8sa` (release), Java doğrulaması `verify-egp3gvx_` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. Vec2 add/dot/distance işlemleri, daha geniş input hareketi ve kamera dönüşümleri ayrı davranış dilimleridir.
