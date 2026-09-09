# P01.b — Vec2 rotate(double) doğrulaması

`src/math.rs`, Java `Vec2.rotate(double)` davranışını f64 açı, Java `Mth` lookup sinüs/kosinüs değerleri ve f32 vektör işlemleriyle karşılar. Java oracle, `Vec2.rotate()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 859 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **859/859** math senaryosunda eşleşti.
- 18 yeni senaryoda temel ve eksen dönüşleri, lookup quantization, signed zero, subnormal, sonlu sınır, Infinity, NaN ve f32 taşması karşılaştırıldı.
- Birleşik altı aile corpus'u **1313/1313** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1313 senaryoluk Java referansını yeniden doğruladı.

Math referansı `b65c658bf8c61413f639524850d054786ff936198bd6d0bb52662da232a61d77`, birleşik referans `5ddc1ee352aaf51a604b52f8e9f1d9356e75f176e3e2bdb0b5a00c406d6c51e2` kimliklerindedir. Math karşılaştırma raporları `compare-_2ksshx8` (debug) ve `compare-z3kwikqe` (release); birleşik raporlar `compare-6n7i4k79` (debug) ve `compare-se85bnok` (release), Java doğrulaması `verify-1_zhkgp3` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. Vec2 add/dot/distance işlemleri, daha geniş input hareketi ve kamera dönüşümleri ayrı davranış dilimleridir.
