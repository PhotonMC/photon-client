# P01.b — Vec2 rotate(double) doğrulaması

`src/math.rs`, Java `Vec2.rotate(double)` davranışını f64 açı, Java `Mth` lookup sinüs/kosinüs değerleri ve f32 vektör işlemleriyle karşılar. Java oracle, `Vec2.rotate()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 965 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **965/965** math senaryosunda eşleşti.
- 18 yeni senaryoda temel ve eksen dönüşleri, lookup quantization, signed zero, subnormal, sonlu sınır, Infinity, NaN ve f32 taşması karşılaştırıldı.
- Birleşik altı aile corpus'u **1467/1467** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1467 senaryoluk Java referansını yeniden doğruladı.

Math referansı `bbc2c1dc1c594cc962e6acbf6fcdc98815c896f66dcad6a72c2da4be5b0a0f81`, birleşik referans `5cb121b3bc1e12571ad171166a785300fd6e581cf78813e4ddaf2b99d19ab7d1` kimliklerindedir. Math karşılaştırma raporları `compare-ctepk6ji` (debug) ve `compare-5wubt_if` (release); birleşik raporlar `compare-lsdsnr89` (debug) ve `compare-b5y_ka7p` (release), Java doğrulaması `verify-o3reu3wd` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. Vec2 add/dot/distance işlemleri, daha geniş input hareketi ve kamera dönüşümleri ayrı davranış dilimleridir.
