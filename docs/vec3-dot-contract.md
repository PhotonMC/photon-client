# P01.b — Vec3 dot sözleşmesi

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](vec3-dot-parity.md)). Kapsam, Java `Vec3.dot(Vec3)` metodunun üç bileşen çarpımını sıralı double toplamı olarak üretmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:87-89` içindeki `dot(Vec3)` incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `dot(Vec3)`, x, y ve z bileşen çiftlerini ayrı ayrı çarpar ve bu üç ürünü Java kaynak sırasıyla toplar.
- Sonuç tek bir f64 değerdir; vektör üzerinde değişiklik yapmaz.
- Çarpım ve toplama sırası ara yuvarlama, overflow ve cancellation sonuçlarını belirler; eşdeğer görünen yeniden sıralama kullanılmaz.
- Signed zero, NaN, Infinity ve taşma sonuçları ham IEEE-754 bitleriyle karşılaştırılır; tolerans veya NaN normalizasyonu uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu, farklı bileşenli iki Vec3 | Dot sonucu |
| `order-and-cancellation` | Double hassasiyetinde küçük terim ve karşıt büyük terimler | Sıralı toplama sonucu |
| `signed-zero` | Negatif sıfır çarpımları | Sonuç işareti |
| `overflow` | En büyük sonlu double ile çarpım | Infinity bitleri |
| `special-values` | Infinity ile sıfır çarpımı | NaN bitleri |

Girdiler `family: "math"`, `operation: "vec3_dot"`, `first` ve `second` Vec3 nesnelerini kullanır. Bileşenler 16 haneli küçük harfli f64 bit string'leridir; sonuç `value` alanında aynı biçimde döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::dot`, üç çarpımı ve toplamı doğrudan kaynak sırasıyla uygular. `examples/behavior_adapter.rs` `vec3_dot` operasyonunu, `tests/behavior/java/MathOracle.java` ise beklenen değeri doğrudan Java `dot` çağrısını kullanarak bağlar.
