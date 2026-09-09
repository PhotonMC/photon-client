# P01.b — Vec3 rotateClockwise90 sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.rotateClockwise90()` metodunun saat yönündeki 90 derece eksen permütasyonunu içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:259-261` içindeki metod incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Sonuç bileşenleri `(−z, y, x)` sırasıyla yeni bir Vec3 içinde oluşturulur.
- `z` bileşeninin unary negation işlemi yalnızca işaret bitini çevirir; y ve x ham f64 bitleriyle taşınır.
- Metod trigonometrik hesap, clamp veya mutasyon yapmaz; Infinity, NaN, subnormal ve signed zero değerleri aynı IEEE-754 işlemiyle gözlenir.
- Başlangıç ve sonuç Vec3 snapshot'ları toleranssız ham f64 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Sonlu pozitif/negatif bileşenler | `(−z, y, x)` permütasyonu |
| `negative-components` | Farklı işaretli sonlu bileşenler | İşaret ve eksen sırası |
| `signed-zero` | Signed-zero bileşenler | Unary negation ile sıfır işareti |
| `subnormal-and-large` | En küçük subnormal ve en büyük sonlu değer | Sınır değerleri ve işaret çevirme |
| `special-values` | Infinity ve NaN bileşenleri | Özel değerlerin doğrudan taşınması |

Girdiler `family: "math"`, `operation: "vec3_rotate_clockwise_90"` ve on altı haneli f64 bit string'lerinden oluşan `first` Vec3 alanını taşır. Sonuç `value` alanında Vec3 snapshot'ıdır; bileşenler, uzunluk ve uzunluk karesi on altı haneli küçük harfli f64 bit string'leridir.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::rotate_clockwise_90`, Java'nın `-z`, `y`, `x` sırasını ve unary negation işaret bitini korur. `examples/behavior_adapter.rs` snapshot döndürür; `tests/behavior/java/MathOracle.java` aynı `rotateClockwise90` çağrısını doğrudan oracle çıktısına bağlar.
