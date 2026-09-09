# P01.b — Vec2 distanceToSqr sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec2.distanceToSqr(Vec2)` metodunun iki f32 bileşen farkını kareleyip toplamasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec2.java:72-76` içindeki `distanceToSqr` gövdesi incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `p.x - this.x` ve `p.y - this.y` farkları f32 olarak ayrı hesaplanır.
- Önce x farkının, sonra y farkının karesi alınır; iki kare kaynak sırasıyla f32 olarak toplanır.
- Signed zero, ara yuvarlama, taşma, Infinity ve NaN sonuçları ham IEEE-754 bitleriyle karşılaştırılır.
- Sonuç tek f32 değeridir; tolerans veya NaN normalizasyonu uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vector-basic` | Sonlu iki Vec2 | Kareli uzaklık |
| `order-and-rounding` | Büyük x farkı ve küçük y farkı | f32 toplama yuvarlaması |
| `signed-zero` | İşaretli sıfır bileşenleri | Kare ve toplam bitleri |
| `overflow` | En büyük sonlu f32 bileşenleri | Infinity bitleri |
| `special-values` | Infinity ve NaN bileşenleri | Özel f32 sonucu |

Girdiler `family: "math"`, `operation: "vec2_distance_to_sqr"`, `first` ve `second` Vec2 nesnelerini kullanır. Bileşenler 8 haneli küçük harfli f32 bit string'leridir; sonuç `value` alanında aynı biçimde döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec2::distance_to_sqr`, Java'nın çıkarma, kare alma ve kaynak sıralı toplama adımlarını uygular. `examples/behavior_adapter.rs` operasyonu, `tests/behavior/java/MathOracle.java` ise beklenen değeri doğrudan Java `distanceToSqr` çağrısıyla bağlar.
