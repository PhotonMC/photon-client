# P01.b — Vec3 eksen dönüşü sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **18 Java davranış testi hazırlanmıştır**. Kapsam `Vec3.xRot`, `Vec3.yRot` ve `Vec3.zRot` yöntemlerinin tek bir f32 radyan girdisiyle yeni Vec3 üretmesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:232-261` ve `util/Mth.java:29-55` incelendi. Oracle doğrudan Java yöntemlerini çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Radyan parametresi f32'dür ve `Mth.sin`/`Mth.cos` lookup tablosuna double olarak genişletilir. Tablo 16-bit açı quantization'ı ve f32 sonuçları korur.
- `xRot`, X bileşenini koruyup Y/Z düzleminde; `yRot`, Y bileşenini koruyup X/Z düzleminde; `zRot`, Z bileşenini koruyup X/Y düzleminde kaynak Java formülleriyle dönüş yapar.
- Sin/cos f32 değerleri Vec3 f64 bileşenleriyle işleme girdiğinde f64'e yükseltilir. Çarpma ve toplama/çıkarma sırası gözlenebilir.
- Signed zero, subnormal/sonlu sınırlar, Infinity, NaN ve NaN/Infinity radyanlarının lookup index davranışı sonuçta görünür.

## Senaryolar ve JSONL

Her girdi `family: "math"`, `operation: "vec3_rotate"`, `axis: "x"|"y"|"z"`, f64 bit dizgeli `first` Vec3 ve f32 bit dizgeli `radians` taşır. Sonuç `value` içinde Vec3 snapshot'ıdır. Temel, negatif açı, küçük quantized açı, çeyrek dönüş, signed zero, özel vektör ve özel açı yolları üç eksen arasında dağıtılmış 18 senaryo ile sınanır.

## Rust karşılığı

Rust karşılığı, Java `Mth` tablosunun üretim ve indexleme sözleşmesini `src/math.rs` içinde koruyup `Vec3` dönüş yöntemlerini aynı f32/f64 sınırında uygular. `examples/behavior_adapter.rs` sonucu taşır; `tests/behavior/java/MathOracle.java` Java `xRot`/`yRot`/`zRot` çağrısını yapar.


## Client-parity doğrulaması

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
