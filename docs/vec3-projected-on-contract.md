# P01.b — Vec3 projectedOn sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.projectedOn(Vec3)` metodunun bir vektörü `onto` vektörüne izdüşürme ve sıfır uzunluk karesi kısa yolunu içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:323-325` içindeki `projectedOn` metodu ile `lengthSqr`, `dot` ve `scale` çağrılarının kaynak sırası incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Önce `onto.lengthSqr() == 0.0` karşılaştırılır; sonuç doğruysa `onto` vektörü yeni işlem yapılmadan aynen döndürülür.
- Sıfır olmayan durumda işlem sırası `onto.scale(this.dot(onto)).scale(1.0 / onto.lengthSqr())` biçimindedir.
- `lengthSqr()` iki kez çağrılır; her iki çağrı da aynı f64 bileşen karesi toplamını kullanır.
- Signed zero, subnormal underflow, overflow, Infinity ve NaN bitleri toleranssız f64 snapshot'larıyla karşılaştırılır.
- Sonuç yeni bir Vec3'tür; sıfır kısa yolunda dönen `onto` bileşenleri de değiştirilmez.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Sonlu vektörün x eksenine izdüşümü | Dot ve iki scale yolunun temel sonucu |
| `orthogonal` | Birbirine dik sonlu vektörler | Sıfır izdüşüm ve signed zero sonucu |
| `zero-onto-preserves` | Signed zero bileşenli sıfır `onto` vektörü | Sıfır uzunluk karesi kısa yolu |
| `negative-fraction` | Negatif ve kesirli sonlu bileşenler | İşaret ve işlem sırası |
| `special-values` | Infinity bileşenleri ve sonlu `onto` | NaN yayılımı |
| `subnormal-onto` | En küçük subnormal `onto` bileşeni | Uzunluk karesi underflow'u ve kısa yol |

Girdiler `family: "math"`, `operation: "vec3_projected_on"`, `first` ve `onto` Vec3 nesnelerinden oluşur. Sonuç `value` içinde Vec3 snapshot'ı olarak döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::projected_on`, önce `onto.length_squared()` değerini Java koşuluyla sınar; devam eden yolda `dot`, iki `scale` ve ters uzunluk karesi sırasını korur. `examples/behavior_adapter.rs` iki Vec3 girdisini ham f64 bitlerinden okur; `tests/behavior/java/MathOracle.java` aynı `Vec3.projectedOn` çağrısını oracle çıktısına bağlar.
