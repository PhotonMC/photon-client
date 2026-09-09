# P01.b — Vec3 eksen dönüşü doğrulaması

`src/math.rs`, Java `Vec3.xRot`, `yRot` ve `zRot` davranışını aynı f32 radyan ve Java `Mth` lookup sözleşmesiyle karşılar. Java oracle, her eksen için özgün yöntemi doğrudan çağırır; beklenen Vec3 snapshot'ı Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1013 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1013/1013** math senaryosunda eşleşti.
- 18 yeni senaryoda üç eksen, negatif ve küçük quantized açılar, çeyrek dönüş, signed zero, özel vektörler ve NaN/Infinity açıları karşılaştırıldı.
- Birleşik altı aile corpus'u **1515/1515** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1515 senaryoluk Java referansını yeniden doğruladı.

Math referansı `5ce242e52e41e1869ca609121bb59c3acf18c2e75ba193e14bf67ead2c12ed7b`, birleşik referans `b6d98de9faec99fa12618268a20692921e10931fc251b87964810964c0de2933` kimliğindedir. Math karşılaştırma raporları `compare-dsrceuzh` (debug) ve `compare-2cvy6vt1` (release); birleşik raporlar `compare-5omvt90q` (debug) ve `compare-orlrlnqo` (release), Java doğrulaması `verify-wa8bpydo` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec3 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. `Vec3.directionFromRotation`, `rotation`, local coordinate dönüşleri ve `Vec2.rotate` ayrı davranış dilimleridir.
