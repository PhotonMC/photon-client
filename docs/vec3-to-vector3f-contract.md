# P01.b — Vec3 toVector3f sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.toVector3f()` metodunun f64 bileşenleri JOML `Vector3f` içindeki f32 bileşenlere dönüştürmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:319-321` içindeki üç ayrı `(float)` dönüşümü ve yeni `Vector3f` kurulumu incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- x, y ve z bileşenleri birbirinden bağımsız olarak Java f64 → f32 daraltmasından geçirilir.
- Sonuç üç f32 bileşenli yeni bir `Vector3f` değeridir; Vec3'ün f64 bileşenleri değişmez.
- IEEE-754 yuvarlama, signed zero, subnormal değerlerin sıfıra veya f32 subnormal'a dönüşmesi, overflow, Infinity ve NaN ham f32 bitleriyle karşılaştırılır.
- Sonuç `Vector3f` olduğundan Vec3'e özgü length ve length squared alanları bu operasyona dahil değildir.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Sonlu, tam temsil edilebilir bileşenler | Üç bağımsız f32 dönüşümü |
| `rounding` | f32 aralığında yuvarlama gerektiren f64 değerler | Java daraltma yuvarlaması |
| `signed-zero` | Pozitif/negatif sıfır ve en küçük f64 subnormal | İşaret bitleri ve sıfıra underflow |
| `subnormal` | f64 subnormal'ları ve f32 en küçük subnormal sınırı | f32 subnormal üretimi |
| `overflow` | Çok büyük pozitif/negatif sonlu değerler | f32 Infinity taşması |
| `special-values` | Infinity ve özel NaN payload'ı | Özel değer işaret/payload sonucu |

Girdiler `family: "math"`, `operation: "vec3_to_vector3f"` ve f64 bit dizgeli `first` Vec3 nesnesinden oluşur. Sonuç `value` içinde f32 `x`, `y`, `z` ham bit snapshot'ı olarak döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::to_vector3f`, üç bileşeni ayrı ayrı `f32`'ye daraltıp `Vec3f` değeri üretir. `examples/behavior_adapter.rs` sonucu f32 ham bitleriyle taşır; `tests/behavior/java/MathOracle.java` aynı `Vec3.toVector3f()` çağrısını oracle çıktısına bağlar.
