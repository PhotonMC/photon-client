# P01.b — Vec2 rotate(double) sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **18 Java davranış testi hazırlanmıştır**. Kapsam `Vec2.rotate(double)` yönteminin f32 vektörü f64 radyan açıyla döndürmesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec2.java:82-86` ve `util/Mth.java:50-55` incelendi. Oracle doğrudan Java yöntemini çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Açı f64 olarak alınır; `Mth.cos` ve `Mth.sin` f64 girdiyi 16-bit sinüs tablosu indeksine çevirerek f32 sonuç döndürür.
- Sonuç x bileşeninde mevcut x ile kosinüs ve mevcut y ile sinüs çarpımlarını çıkarma sırasıyla; y bileşeninde mevcut y ile kosinüs ve mevcut x ile sinüs çarpımlarını toplama sırasıyla hesaplar.
- Çarpma, toplama ve çıkarma f32 olarak gerçekleşir. Dönüş yeni `Vec2` içindedir; snapshot x/y yanında f32 `length` ve `lengthSquared` sonuçlarını taşır.
- Sıfır ve signed zero, subnormal, lookup sınırları, büyük sonlu açı, Infinity, NaN ve vektör taşması özel değerleri bit düzeyinde korunur.

## Senaryolar ve JSONL

Her girdi `family: "math"`, `operation: "vec2_rotate"`, f32 bit dizgeli `first` Vec2 ve f64 bit dizgeli `angle` taşır. Temel, işaretli bileşen, eksen dönüşleri, quantized küçük açı, tam tur, signed zero, subnormal, sonlu sınır, özel vektör ve özel açı yolları 18 özgün sentetik senaryo ile sınanır.

## Rust karşılığı

Rust karşılığı `crates/photon-core/src/math.rs` içindeki `Vec2::rotate` yordamında Java `Mth` lookup yardımcılarını ve f32 işlem sırasını korur. `examples/behavior_adapter.rs` sonucu taşır; `tests/behavior/java/MathOracle.java` doğrudan Java `Vec2.rotate()` çağrısını yapar.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Vec2.rotate(double)` davranışını f64 açı, Java `Mth` lookup sinüs/kosinüs değerleri ve f32 vektör işlemleriyle karşılar. Java oracle, `Vec2.rotate()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- 18 yeni senaryoda temel ve eksen dönüşleri, lookup quantization, signed zero, subnormal, sonlu sınır, Infinity, NaN ve f32 taşması karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. Vec2 add/dot/distance işlemleri, daha geniş input hareketi ve kamera dönüşümleri ayrı davranış dilimleridir.
