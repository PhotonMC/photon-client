# P01.b — Vec3 rotation() sözleşmesi

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

Rust karşılığı `src/math.rs` içindeki `Vec3::rotation` yordamında f64 `atan2`/`asin`/`sqrt` hesap sırasını, f32 açı daraltmalarını, derece sabitini ve Vec2 snapshot aritmetiğini korur. `examples/behavior_adapter.rs` sonucu taşır; `tests/behavior/java/MathOracle.java` doğrudan Java `Vec3.rotation()` çağrısını yapar.
