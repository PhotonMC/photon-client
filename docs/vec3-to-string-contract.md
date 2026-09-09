# P01.b — Vec3 toString sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, `Vec3.toString()` sonucunun üç double bileşeni ve sabit ayraçlarla biçimlendirmesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:224-226` içindeki `toString` gövdesi incelendi. Oracle doğrudan Java metodunu çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Sonuç `(x, y, z)` biçimindedir; parantez, virgül ve boşluklar sabittir.
- Her bileşen Java `Double.toString` kurallarıyla yazılır. Ondalık/bilimsel gösterim eşiği, `.0`, signed zero, `Infinity`, `-Infinity` ve `NaN` metinleri gözlenebilir.
- `toString()` vektörün bileşenlerini değiştirmez.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Sonlu ve asimetrik bileşenler | Sabit metin biçimi ve temel double yazımı |
| `scientific` | Küçük/büyük değerler ve signed zero | Bilimsel gösterim ve işaretli sıfır |
| `thresholds` | `1e-3`, `1e7` komşuları | Java double gösterim eşikleri |
| `signed-zero` | Pozitif ve negatif sıfır | Sıfır işaretinin korunması |
| `infinity` | Pozitif/negatif Infinity | Sonsuzluk metinleri |
| `nan` | Farklı NaN bit desenleri | NaN metni ve payload bağımsızlığı |

Girdiler `family: "math"`, `operation: "vec3_to_string"` ve f64 bit dizgeli `first` Vec3 nesnesinden oluşur. Sonuç, `value` alanında Java metni olarak taşınır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::to_string_java`, Java bileşen biçimini ortak `java_double_to_string` yordamıyla üretir. Aynı yordam AABB ve VoxelShape metinlerinde de kullanılır; `examples/behavior_adapter.rs` sonucu taşır, `tests/behavior/java/MathOracle.java` Java `Vec3.toString()` metodunu çağırır.
