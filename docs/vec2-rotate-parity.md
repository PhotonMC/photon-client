# P01.b — Vec2 rotate(double) doğrulaması

`src/math.rs`, Java `Vec2.rotate(double)` davranışını f64 açı, Java `Mth` lookup sinüs/kosinüs değerleri ve f32 vektör işlemleriyle karşılar. Java oracle, `Vec2.rotate()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 923 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **923/923** math senaryosunda eşleşti.
- 18 yeni senaryoda temel ve eksen dönüşleri, lookup quantization, signed zero, subnormal, sonlu sınır, Infinity, NaN ve f32 taşması karşılaştırıldı.
- Birleşik altı aile corpus'u **1425/1425** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1425 senaryoluk Java referansını yeniden doğruladı.

Math referansı `e11b3ae14a5bdb99014f3283b0ed617e1644a4e15c06be01af7f8b8ef84743d6`, birleşik referans `46678aa48c299841b0c8c187569d7f3358ea4c2098c365f4570a69310eeb7422` kimliklerindedir. Math karşılaştırma raporları `compare-85j8zrrj` (debug) ve `compare-eq8g_l1t` (release); birleşik raporlar `compare-ufi9ykko` (debug) ve `compare-fuqao9g1` (release), Java doğrulaması `verify-sqpfj18l` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. Vec2 add/dot/distance işlemleri, daha geniş input hareketi ve kamera dönüşümleri ayrı davranış dilimleridir.
