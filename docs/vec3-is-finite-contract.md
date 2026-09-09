# P01.b — Vec3 isFinite sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.isFinite()` metodunun üç f64 bileşenini sonluluk açısından sorgulamasını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:347-349` içindeki metod incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Sonuç, x, y ve z bileşenlerinin üçünün de `Double.isFinite` olması durumunda `true` olur.
- Pozitif veya negatif Infinity ve her NaN değeri sonucu `false` yapar.
- Sonlu en büyük değer, subnormal değer ve signed zero sonludur; bu değerler `true` sonucunu korur.
- Metod bileşenleri değiştirmez ve sayısal hesap yapmaz; sonuç toleranssız boolean olarak karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `all-finite` | En büyük sonlu değer, signed subnormal ve pozitif sıfır | Sonlu sınırlar ve signed zero |
| `positive-infinity` | x bileşeninde pozitif Infinity | Pozitif sonsuzluk reddi |
| `negative-infinity` | y bileşeninde negatif Infinity | Negatif sonsuzluk reddi |
| `nan` | z bileşeninde payload taşıyan NaN | NaN reddi |
| `mixed-special-values` | Infinity, NaN ve subnormal karışımı | Birden fazla özel değer |

Girdiler `family: "math"`, `operation: "vec3_is_finite"` ve on altı haneli f64 bit string'lerinden oluşan `first` Vec3 alanını taşır. Sonuç `value` alanında boolean olarak döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::is_finite`, x, y ve z için Rust f64 sonluluk sorgularını Java'nın üçlü koşuluyla aynı boolean sonucu verecek şekilde bağlar. `examples/behavior_adapter.rs` f64 bit girdilerini çözer; `tests/behavior/java/MathOracle.java` aynı `isFinite` çağrısını doğrudan oracle çıktısına bağlar.
