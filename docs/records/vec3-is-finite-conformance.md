# P01.b — Vec3 isFinite sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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

`crates/photon-core/src/math.rs` içindeki `Vec3::is_finite`, x, y ve z için Rust f64 sonluluk sorgularını Java'nın üçlü koşuluyla aynı boolean sonucu verecek şekilde bağlar. `examples/behavior_adapter.rs` f64 bit girdilerini çözer; `tests/behavior/java/MathOracle.java` aynı `isFinite` çağrısını doğrudan oracle çıktısına bağlar.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Vec3.isFinite()` metodunu karşılar. Rust adaptörü üç f64 bileşenini sonluluk için sorgular; Java oracle doğrudan `Vec3.isFinite` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu sınırlar, signed zero, pozitif/negatif Infinity, payload taşıyan NaN ve karışık özel değerler karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.isFinite` sonucunun boolean çıktısıyla sınırlıdır. Java `Double.isFinite` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
