# P01.b — Vec3 with sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.with(Direction.Axis, double)` metodunun seçilen bileşeni yeni değerle değiştirip diğer bileşenleri korumasını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:292-297` içindeki `with` metodu ve `core/Direction.java:402-462` içindeki eksen değerleri incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `Axis.X`, yalnızca x bileşenini; `Axis.Y`, yalnızca y bileşenini; `Axis.Z`, yalnızca z bileşenini verilen f64 değerle değiştirir.
- Değiştirilmeyen bileşenlerin IEEE-754 bitleri aynen korunur.
- Yeni değer aritmetik veya normalizasyondan geçmez; signed zero, subnormal, sonlu sınır, Infinity ve NaN bitleri doğrudan taşınır.
- Sonuç yeni bir Vec3'tür; giriş vektörü değiştirilmez ve tüm bileşenler ham bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `x` | Sonlu bileşenler, `axis: x`, yeni sonlu değer | X bileşeni değişimi |
| `y-signed-zero` | Signed zero ve büyük sonlu değer, `axis: y`, negatif sıfır | Yeni signed zero bitinin korunması |
| `z-subnormal` | Sonlu bileşenler, `axis: z`, en küçük pozitif subnormal | Z bileşeni ve subnormal değer |
| `x-special-value` | Infinity, payload taşıyan NaN ve negatif Infinity, `axis: x`, NaN payload | Özel yeni değerin ham bitleri |
| `y-max-finite` | Subnormal ve signed zero bileşenleri, `axis: y`, en büyük sonlu değer | Korunan bileşen bitleri ve sonlu sınır |

Girdiler `family: "math"`, `operation: "vec3_with"`, `first` Vec3 nesnesi, `axis` adı ve 16 haneli f64 bit string'i `value` alanından oluşur. Sonuç `value` içinde Vec3 snapshot'ı olarak döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::with`, seçilen `Axis` için yalnızca ilgili bileşeni değiştirerek yeni Vec3 üretir. `examples/behavior_adapter.rs` ekseni `Axis::from_name` ile çözer ve f64 değerini ham bitlerden okur; `tests/behavior/java/MathOracle.java` aynı `Vec3.with` çağrısını doğrudan oracle çıktısına bağlar.
