# P01.b — Vec3 directionFromRotation sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **18 Java davranış testi hazırlanmıştır**. Kapsam `Vec3.directionFromRotation(float, float)` ve `Vec3.directionFromRotation(Vec2)` overload'larının pitch/yaw derecelerinden yeni Vec3 üretmesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:267-279`, `world/phys/Vec2.java:18-27` ve `util/Mth.java:29-55` incelendi. Oracle doğrudan Java overload'larını çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `float, float` overload'u pitch (`rotX`) ve yaw (`rotY`) değerlerini Java'nın f32 derece→radyan sabitiyle f32 ara işlemlerinde dönüştürür.
- `Vec2` overload'u `rotation.x` ve `rotation.y` f32 bileşenlerini aynı `float, float` yoluna verir.
- Dönüşümde `Mth.sin`/`Mth.cos` 16-bit lookup quantization'ı ve f32 sonuçları korunur. Son üç yön bileşeni f32 çarpımları olarak hesaplanır ve Vec3'e f64 olarak genişletilir.
- Sıfır, signed zero, küçük açı, 90/180 derece, büyük açı, NaN, Infinity ve f32 taşma sınırları lookup index ve çıktı bitleriyle gözlenebilir.

## Senaryolar ve JSONL

Her girdi `family: "math"`, `operation: "vec3_direction_from_rotation"` ve `kind: "components"` veya `"vec2"` taşır. İlk biçim f32 bit dizgeli `rot_x`/`rot_y`, ikinci biçim f32 bit dizgeli `rotation` Vec2 kullanır. Sonuç `value` içinde Vec3 snapshot'ıdır. Temel, cardinal pitch/yaw, birleşik ve sarılmış açılar, küçük quantized açı, signed zero ve özel açı yolları iki overload arasında dağıtılmış 18 senaryo ile sınanır.

## Rust karşılığı

Rust karşılığı `src/math.rs` içindeki iki `Vec3` yordamında açı dönüşümünü, Java f32 ara işlemlerini, Mth lookup yolunu ve f32 ürünlerini korur. `examples/behavior_adapter.rs` iki JSON biçimini üretim API'sine taşır; `tests/behavior/java/MathOracle.java` Java overload'larını doğrudan çağırır.


## Client-parity doğrulaması

`src/math.rs`, Java `Vec3.directionFromRotation(float, float)` ve `directionFromRotation(Vec2)` overload'larını aynı f32 derece→radyan, Java `Mth` lookup ve f32 ürün sınırlarıyla karşılar. Java oracle, iki overload'u doğrudan çağırır; beklenen Vec3 snapshot'ı Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- 18 yeni senaryoda iki overload, cardinal ve birleşik açıları, quantization, signed zero ve NaN/Infinity/taşma açılarını karşılaştırdı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec3 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. [`Vec3.rotation()`](vec3-rotation-method-conformance.md), local coordinate dönüşleri ve `Vec2.rotate` ayrı davranış dilimleridir.
