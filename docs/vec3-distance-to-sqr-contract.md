# P01.b — Vec3 distanceToSqr sözleşmesi

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](vec3-distance-to-sqr-parity.md)). Kapsam, Java `Vec3.distanceToSqr` metodunun Vec3 ve üç bileşenli overload'larını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:131-141` içindeki `distanceToSqr` overload'ları incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Vec3 overload'ı hedef vektörün her bileşeninden mevcut vektör bileşenini çıkarır; üç bileşenli overload aynı işlemi verilen hedef koordinatlarıyla yapar.
- Farklar ayrı ayrı karesine alınır ve x, y, z terimleri Java kaynak sırasıyla toplanır; karekök alınmaz.
- Girdi vektörü değişmez ve sonuç tek bir f64 değerdir.
- Signed zero, NaN, Infinity ve double taşması ham IEEE-754 bitleriyle karşılaştırılır; tolerans veya NaN normalizasyonu uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vector-basic` | Mevcut ve hedef Vec3 | Vec3 overload sonucu |
| `components-basic` | Mevcut Vec3 ve üç hedef bileşeni | Üç bileşenli overload sonucu |
| `order-and-rounding` | Büyük ve küçük fark kareleri | Kaynak toplama sırası sonucu |
| `signed-zero` | İşaretli sıfır farkları | Pozitif sıfır sonucu |
| `overflow-and-special-values` | En büyük sonlu double, Infinity ve NaN | Taşma/özel değer bitleri |

Girdiler `family: "math"` kullanır. Vec3 overload'ı `operation: "vec3_distance_to_sqr"` ile `first` ve `second` nesnelerini, üç bileşenli overload `operation: "vec3_distance_to_sqr_components"` ile `first`, `x`, `y`, `z` alanlarını taşır. Tüm f64 girdileri ve sonuç `value` alanı 16 haneli küçük harfli bit string'leridir.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::distance_to_sqr` ve `distance_to_sqr_components`, fark karelerini doğrudan kaynak sırasıyla toplar. `examples/behavior_adapter.rs` iki JSONL operasyonunu, `tests/behavior/java/MathOracle.java` ise beklenen değerleri doğrudan Java overload çağrılarına bağlar.
