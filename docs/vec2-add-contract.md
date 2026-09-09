# P01.b — Vec2 add overload'ları sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec2.add(Vec2)` ve `Vec2.add(float)` overload'larının f32 bileşen toplamasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec2.java:37-43` içindeki iki `add` overload'u incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `add(Vec2)`, x bileşenlerini ve y bileşenlerini ayrı ayrı f32 olarak toplar.
- `add(float)`, aynı scalar değeri iki bileşene de ayrı ayrı ekler.
- Her bileşen kaynakta yazıldığı sırayla hesaplanır; sonuç yeni Vec2 içindedir.
- Ara yuvarlama, signed zero, taşma, Infinity ve NaN sonuçları ham IEEE-754 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vector-basic` | Sonlu iki Vec2 | Vec2 toplamı |
| `vector-signed-zero` | Negatif sıfır bileşenleri | Sonuç işaretleri |
| `vector-overflow` | En büyük sonlu f32 bileşenleri | Infinity bitleri |
| `scalar-basic` | Sonlu Vec2 ve scalar | Her bileşene scalar eklenmesi |
| `scalar-special-values` | Infinity ve negatif Infinity | NaN/Infinity bitleri |

Girdiler `family: "math"`, `operation: "vec2_add_vec2"` veya `operation: "vec2_add_scalar"` kullanır. Vec2 bileşenleri 8 haneli küçük harfli f32 bit string'leridir; sonuç `value` alanında Vec2 snapshot'ı olarak döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec2::add_vec2` ve `Vec2::add_scalar`, Java overload'larının bileşen işlem sırasını uygular. `examples/behavior_adapter.rs` iki operasyonu, `tests/behavior/java/MathOracle.java` ise beklenen değerleri doğrudan Java `add` çağrılarıyla bağlar.
