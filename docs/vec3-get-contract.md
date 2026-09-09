# P01.b — Vec3 get sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.get(Direction.Axis)` metodunun seçilen eksenin ham f64 bileşenini döndürmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:288` ve `core/Direction.java:402-462,541` içindeki `get` ve `Axis.choose(double, double, double)` yolları incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `Axis.X`, `x`; `Axis.Y`, `y`; `Axis.Z`, `z` bileşenini doğrudan seçer.
- Seçim aritmetik, normalizasyon veya dönüşüm yapmaz; seçilen değerin IEEE-754 f64 bitleri korunur.
- Signed zero, subnormal, sonlu sınır, Infinity ve NaN seçilebilir değerlerdir ve ham bitleriyle karşılaştırılır.
- Vektör değiştirilmez; sonuç toleranssız 16 haneli f64 bit string'i olarak taşınır.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `x` | Sonlu ve işaretli bileşenler, `axis: x` | X bileşeni seçimi |
| `y` | Negatif ve pozitif sonlu bileşenler, `axis: y` | Y bileşeni seçimi |
| `z-signed-zero` | Pozitif ve negatif sıfırlar, `axis: z` | Signed zero bitinin korunması |
| `subnormal-and-large` | Subnormal, negatif subnormal ve en büyük sonlu değer, `axis: z` | Sınır değerlerin aritmetiksiz seçimi |
| `special-values` | Infinity, payload taşıyan NaN ve negatif Infinity, `axis: y` | Özel f64 bitlerinin korunması |

Girdiler `family: "math"`, `operation: "vec3_get"`, `first` Vec3 nesnesi ve `axis` olarak taşınır. Bileşenler 16 haneli küçük harfli f64 bit string'leridir; sonuç `value` alanında aynı biçimde döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::get`, `Axis` eşleşmesiyle x/y/z bileşenini doğrudan döndürür. `examples/behavior_adapter.rs` ekseni mevcut `Axis::from_name` yoluyla çözer ve sonucu bit string olarak taşır; `tests/behavior/java/MathOracle.java` aynı `Vec3.get` çağrısını doğrudan oracle çıktısına bağlar.
