# P01.b — Vec2 dot sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec2.dot(Vec2)` metodunun iki f32 bileşen çarpımını kaynak sırasıyla toplamasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec2.java:33-35` içindeki `dot(Vec2)` incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- x bileşenleri ve y bileşenleri ayrı ayrı f32 olarak çarpılır.
- İki ürün kaynakta yazıldığı sırayla f32 toplama işlemine verilir.
- Sonuç tek bir f32 değerdir; giriş vektörleri değişmez.
- Ara yuvarlama, cancellation, taşma, signed zero, Infinity ve NaN sonuçları ham IEEE-754 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu, farklı bileşenli iki Vec2 | Dot sonucu |
| `order-and-rounding` | Küçük f32 terim ile yuvarlama sınırı | Kaynak sıralı toplam |
| `signed-zero` | Negatif sıfır çarpımları | Sonuç işareti |
| `overflow` | En büyük sonlu f32 ile çarpım | Infinity bitleri |
| `special-values` | Infinity ile sıfır çarpımı | NaN bitleri |

Girdiler `family: "math"`, `operation: "vec2_dot"`, `first` ve `second` Vec2 nesnelerini kullanır. Bileşenler 8 haneli küçük harfli f32 bit string'leridir; sonuç `value` alanında aynı biçimde döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec2::dot`, iki f32 çarpımını ve kaynak sıralı toplamı uygular. `examples/behavior_adapter.rs` `vec2_dot` operasyonunu, `tests/behavior/java/MathOracle.java` ise beklenen değeri doğrudan Java `dot` çağrısını kullanarak bağlar.
