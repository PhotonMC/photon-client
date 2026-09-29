# P01.b — Vec3 rotation() sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **18 Java davranış testi hazırlanmıştır**. Kapsam `Vec3.rotation()` yönteminin f64 Vec3 bileşenlerinden f32 pitch/yaw içeren bir `Vec2` üretmesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:275-279`, `world/phys/Vec2.java:18-27,64-70` ve `util/Mth.java:58-59` incelendi. Oracle doğrudan Java yöntemini çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Yaw, `-x` ve `z` ile f64 `Math.atan2` sonucunun f32'ye daraltılmasından sonra Java'nın f32 `180.0F / (float)Math.PI` sabitiyle çarpılır.
- Pitch, f64 bileşen kareleri toplamının f64 kareköküne bölünen `-y` değerinin f64 `Math.asin` sonucunun f32'ye daraltılmasından sonra aynı f32 derece sabitiyle çarpılmasıyla üretilir.
- Dönüş `Vec2(pitch, yaw)` biçimindedir; sonuç snapshot'ı x/y yanında Vec2'nin f32 `length` ve `lengthSquared` değerlerini taşır.
- Sıfır uzunluk, signed zero, subnormal, sonlu sınırlar, Infinity ve NaN durumlarında `atan2`, `asin`, f64 ara işlem sırası ve Vec2 f32 NaN toplama davranışı korunur. Java `Math.asin` alan dışı girdide negatif canonical NaN döndürür.

## Senaryolar ve JSONL

Her girdi `family: "math"`, `operation: "vec3_rotation"` ve f64 bit dizgeli `first` Vec3 taşır. Sonuç `value` içinde Vec2 snapshot'ıdır. Temel, eksen, birleşik, non-unit, sıfır/signed-zero, subnormal, en büyük sonlu, Infinity ve NaN yolları 18 özgün sentetik senaryo ile sınanır.

## Rust karşılığı

Rust karşılığı `crates/voxel-core/src/math.rs` içindeki `Vec3::rotation` yordamında f64 `atan2`/`asin`/`sqrt` hesap sırasını, f32 açı daraltmalarını, derece sabitini ve Vec2 snapshot aritmetiğini korur. `examples/behavior_adapter.rs` sonucu taşır; `tests/behavior/java/MathOracle.java` doğrudan Java `Vec3.rotation()` çağrısını yapar.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Vec3.rotation()` davranışını f64 trigonometrik ara işlemleri, f32 açı dönüşümü ve Vec2 snapshot hesaplarıyla karşılar. Java oracle, `Vec3.rotation()` yöntemini doğrudan çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- 18 yeni senaryoda eksenler, birleşik ve non-unit vektörler, sıfır/signed-zero, subnormal, sonlu sınırlar ve Infinity/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt 18 özgün sentetik girdinin Java Vec2 snapshot'larına bit düzeyinde eşitliğiyle sınırlıdır. `Vec3.directionFromRotation`, `Vec2.rotate`, kamera yönlendirmesi ve local coordinate dönüşleri ayrı davranış dilimleridir.
