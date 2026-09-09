# P01.b — Vec2 negated sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec2.negated()` metodunun her f32 bileşenini unary negation ile tersleyip yeni Vec2 oluşturmasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec2.java:78-80` içindeki `negated` gövdesi incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Sonuç yeni bir Vec2'dür; x ve y bileşenleri kaynak sırayla unary f32 negation'dan geçirilir.
- Negation yalnızca sign bitini tersler; sonlu değerlerin büyüklüğü, subnormal bitleri ve Infinity korunur.
- Signed zero işaret değiştirir.
- NaN payload bitleri korunurken NaN sign bit'i terslenir; sonuçlar ham IEEE-754 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu pozitif ve negatif bileşenler | İşareti çevrilmiş Vec2 |
| `signed-zero` | Negatif ve pozitif sıfır | Çevrilmiş sıfır işaretleri |
| `subnormal` | Pozitif ve negatif en küçük subnormal | Payload korunarak çevrilmiş bitler |
| `infinity-and-boundary` | En büyük sonlu değer ve negatif Infinity | Negatif sınır ve pozitif Infinity |
| `nan-payload` | Farklı sign/payload NaN değerleri | Ters sign, korunmuş payload |

Girdiler `family: "math"`, `operation: "vec2_negated"` ve `first` Vec2 nesnesini kullanır. Bileşenler 8 haneli küçük harfli f32 bit string'leridir; sonuç `value` alanında Vec2 snapshot'ı olarak döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec2::negated`, Java'nın iki unary f32 negation adımını uygular. `examples/behavior_adapter.rs` operasyonu, `tests/behavior/java/MathOracle.java` ise beklenen değeri doğrudan Java `negated` çağrısıyla bağlar.
