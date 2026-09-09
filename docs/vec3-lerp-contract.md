# P01.b — Vec3 lerp sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.lerp(Vec3, double)` metodunun üç bileşeni için `Mth.lerp(double, double, double)` çağrılarını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:228-230` içindeki metod ve `util/Mth.java:554-560` içindeki Vec3/scalar lerp overload'ları incelendi. Oracle doğrudan Java çağrılarını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Her bileşen f64 hassasiyetinde `p0 + alpha * (p1 - p0)` sırasıyla hesaplanır.
- `Vec3.lerp` x, y ve z bileşenleri için aynı alpha'yı sırayla uygular; alpha 0–1 aralığına clamp edilmez.
- Negatif ve 1.0'dan büyük alpha değerleri extrapolation üretir; signed zero, subnormal, Infinity ve NaN ara sonuçları değiştirilmeden taşınır.
- Başlangıç, hedef, alpha ve sonuç Vec3 snapshot'ları toleranssız ham f64 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Sonlu Vec3'ler, alpha `0.25` | Üç bileşen ve temel lerp sırası |
| `negative-extrapolation` | Sonlu Vec3'ler, alpha `-0.5` | Clamp edilmemiş negatif extrapolation |
| `over-one` | Büyük/küçük sonlu bileşenler, alpha `1.5` | 1.0 üstü extrapolation ve double aritmetiği |
| `signed-zero-and-subnormal` | Signed-zero/subnormal bileşenler, alpha `-0.0` | Sıfır işaretleri ve en küçük f64 değerleri |
| `special-values` | Infinity/NaN bileşenleri, alpha `0.5` | Özel değer yayılımı |

Girdiler `family: "math"`, `operation: "vec3_lerp"`, `first`, `second` ve on altı haneli f64 bit string'i `alpha` alanlarını taşır. Sonuç `value` alanında Vec3 snapshot'ıdır; bileşenler, uzunluk ve uzunluk karesi on altı haneli küçük harfli f64 bit string'leridir.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::lerp`, üç bileşende Java'nın çıkarma, çarpma ve toplama sırasını korur. `examples/behavior_adapter.rs` f64 bit girdilerini çözüp snapshot döndürür; `tests/behavior/java/MathOracle.java` aynı `Vec3.lerp` çağrısını doğrudan oracle çıktısına bağlar.
