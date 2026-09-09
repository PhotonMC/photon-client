# P01.b — Vec3 eksen dönüşü doğrulaması

`src/math.rs`, Java `Vec3.xRot`, `yRot` ve `zRot` davranışını aynı f32 radyan ve Java `Mth` lookup sözleşmesiyle karşılar. Java oracle, her eksen için özgün yöntemi doğrudan çağırır; beklenen Vec3 snapshot'ı Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 941 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **941/941** math senaryosunda eşleşti.
- 18 yeni senaryoda üç eksen, negatif ve küçük quantized açılar, çeyrek dönüş, signed zero, özel vektörler ve NaN/Infinity açıları karşılaştırıldı.
- Birleşik altı aile corpus'u **1443/1443** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1443 senaryoluk Java referansını yeniden doğruladı.

Math referansı `7aaf16bdfc85f793bd25e0d4d234f6988f30874d554dc65c32ce3d0ec4f85363`, birleşik referans `998945022ecb5aa3b303b59cbf3789de5cd227359a784553dadb87da8ba02eb6` kimliğindedir. Math karşılaştırma raporları `compare-9rd6f2k4` (debug) ve `compare-azcslb8q` (release); birleşik raporlar `compare-9epmwbb2` (debug) ve `compare-vdfamuam` (release), Java doğrulaması `verify-povrla2w` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec3 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. `Vec3.directionFromRotation`, `rotation`, local coordinate dönüşleri ve `Vec2.rotate` ayrı davranış dilimleridir.
