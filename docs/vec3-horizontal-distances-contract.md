# P01.b — Vec3 horizontalDistance sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.horizontalDistance()` ve `Vec3.horizontalDistanceSqr()` metodlarını birlikte içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:188-196` içindeki iki gövde incelendi. Oracle doğrudan Java çağrılarını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `horizontalDistanceSqr()` x ve z bileşenlerini ayrı ayrı karesine alır ve `x * x + z * z` kaynak sırasıyla toplar.
- `horizontalDistance()` aynı x/z toplamının `Math.sqrt` sonucunu döndürür.
- y bileşeni iki metodun hesabına girmez; girdi Vec3 değişmez.
- Signed zero, subnormal, büyük sonlu ve Infinity sonuçları ham IEEE-754 f64 bitleriyle karşılaştırılır; tolerans uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu x/y/z bileşenleri | Yatay uzaklık ve karesi |
| `order-and-rounding` | Büyük x ve küçük z karesi | x/z toplama sırası |
| `signed-zero` | İşaretli sıfır x/z bileşenleri | Uzaklık ve kareli uzaklık sıfır bitleri |
| `subnormal` | En küçük subnormal x/z bileşenleri | Kare alma underflow sonucu |
| `infinities` | Sonsuz x/z ve kullanılmayan NaN y | Sonsuzluk sonuçları; y'nin dışlanması |

Girdiler `family: "math"`, `operation: "vec3_horizontal_distances"` ve `first` Vec3 nesnesini taşır. Sonuç `distance` ve `distance_squared` alanlarında 16 haneli küçük harfli f64 bit string'leri bulunur.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::horizontal_distance` ve `Vec3::horizontal_distance_squared`, Java'nın x/z işlem sırasını ve karekök adımını taşır. `examples/behavior_adapter.rs` JSONL operasyonunu, `tests/behavior/java/MathOracle.java` ise iki beklenen değeri doğrudan Java çağrılarına bağlar.
