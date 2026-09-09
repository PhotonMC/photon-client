# P01.b — Vec2 rotate(double) sözleşmesi

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

Rust karşılığı `src/math.rs` içindeki `Vec2::rotate` yordamında Java `Mth` lookup yardımcılarını ve f32 işlem sırasını korur. `examples/behavior_adapter.rs` sonucu taşır; `tests/behavior/java/MathOracle.java` doğrudan Java `Vec2.rotate()` çağrısını yapar.
