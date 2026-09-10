# P01.b — Vec3 eksen dönüşü doğrulaması

`src/math.rs`, Java `Vec3.xRot`, `yRot` ve `zRot` davranışını aynı f32 radyan ve Java `Mth` lookup sözleşmesiyle karşılar. Java oracle, her eksen için özgün yöntemi doğrudan çağırır; beklenen Vec3 snapshot'ı Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- 18 yeni senaryoda üç eksen, negatif ve küçük quantized açılar, çeyrek dönüş, signed zero, özel vektörler ve NaN/Infinity açıları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec3 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. `Vec3.directionFromRotation`, `rotation`, local coordinate dönüşleri ve `Vec2.rotate` ayrı davranış dilimleridir.
