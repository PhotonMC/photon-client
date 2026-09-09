# P01.b — Vec3 vectorTo sözleşmesi

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](vec3-vector-to-parity.md)). Kapsam, Java `Vec3.vectorTo` metodunun mevcut vektörden hedef vektöre yönlü fark üretmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:79-81` içindeki `vectorTo(Vec3)` incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `vectorTo(Vec3)` hedef vektörün her bileşeninden mevcut vektörün aynı bileşenini çıkarır ve yeni bir `Vec3` üretir.
- Çıkarma operand sırası hedef eksi mevcut şeklindedir; sonuç bileşenleri f64 aritmetiğinin Java'daki bit sonucunu taşır.
- Mevcut ve hedef vektörler değişmez; ardışık çağrılarda önceki sonuç sonraki çağrının mevcut vektörüdür.
- Signed zero, NaN, Infinity ve taşma sonuçları için tolerans veya NaN normalizasyonu uygulanmaz.
- Her senaryo başlangıç ve her adım sonrası `x`, `y`, `z`, `length`, `length_squared` bitlerini kaydeder.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu mevcut ve hedef vektörler | Fark vektörü ve türetilmiş ölçümler |
| `chain` | İki vectorTo ve sonraki add/normalize adımları | Sıralı vektör snapshot'ları |
| `signed-zero` | İşaretli sıfır mevcut/hedef bileşenleri | Bileşen bitleri |
| `overflow` | En büyük sonlu double ve ters işaretli hedefler | Infinity ve vektör bitleri |
| `special-values` | Infinity ve NaN hedef bileşenleri | Özel değer bitleri |

Girdiler `family: "math"`, `operation: "vec3"` ve `steps` içinde `vector_to` kullanır. Hedef vektör `target` nesnesinde, bileşenler 16 haneli küçük harfli f64 bit string'leri olarak taşınır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::vector_to`, hedef bileşenlerinden mevcut bileşenleri doğrudan çıkarır. `examples/behavior_adapter.rs` bu metodu JSONL `vector_to` adımına bağlar; `tests/behavior/java/MathOracle.java` beklenen sonucu doğrudan Java `vectorTo` çağrısından alır.
