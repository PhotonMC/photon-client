# P01.b — Vec3 distanceTo sözleşmesi

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](vec3-distance-to-parity.md)). Kapsam, Java `Vec3.distanceTo(Vec3)` metodunun tek overload'ıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:124-129` içindeki `distanceTo` gövdesi incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Hedef vektörün her bileşeninden mevcut vektör bileşeni çıkarılır: `xd = target.x - this.x` ve aynı işlem y/z için yapılır.
- Farklar ayrı ayrı karesine alınır, x, y, z terimleri kaynak sırasıyla toplanır ve toplamın `Math.sqrt` sonucu döndürülür.
- Girdi vektörleri değişmez ve sonuç tek bir f64 değerdir; kareli uzaklığa ayrı bir API çağrısı üzerinden yönlendirme sözleşmenin parçası değildir.
- Signed zero, subnormal, NaN, Infinity ve double taşması ham IEEE-754 bitleriyle karşılaştırılır; tolerans veya NaN normalizasyonu uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu mevcut ve hedef Vec3 | `distanceTo` sonucu |
| `order-and-rounding` | Büyük ve küçük fark kareleri | Kaynak toplama sırası ve karekök sonucu |
| `signed-zero` | İşaretli sıfır uçları | Pozitif sıfır sonucu |
| `subnormal` | En küçük subnormal farklar | Kare alma ve karekök underflow sonucu |
| `overflow-and-special-values` | En büyük sonlu double, Infinity ve NaN | Taşma/özel değer bitleri |

Girdiler `family: "math"` ve `operation: "vec3_distance_to"` kullanır. `first` ve `second` Vec3 nesnelerindeki tüm f64 bileşenleri ve sonuç `value` alanı 16 haneli küçük harfli bit string'leridir.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::distance_to`, fark değişkenlerini doğrudan kaynak sırasıyla hesaplar ve toplamı `sqrt` ile sonlandırır. `examples/behavior_adapter.rs` JSONL operasyonunu, `tests/behavior/java/MathOracle.java` ise beklenen değeri doğrudan Java overload çağrısına bağlar.
