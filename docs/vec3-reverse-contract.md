# P01.b — Vec3 reverse sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.reverse()` metodudur.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:156-158` içindeki `reverse` gövdesi incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `reverse()` ortak ölçek olarak `-1.0` vererek Java `scale` yolunu çağırır.
- Yeni Vec3'ün her bileşeni mevcut bileşenin `-1.0` ile çarpılmasıdır; işlem sırası ve ham IEEE-754 f64 bitleri korunur.
- Girdi vektörü değişmez ve her adımın sonucu sonraki adımın girdisi olur.
- Signed zero, subnormal, büyük sonlu ve Infinity sonuçları toleranssız karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu üç bileşen | Bileşenlerin işaret çevirmesi |
| `chain` | Reverse sonrası bileşen işlemi | Sıralı snapshot'lar |
| `signed-zero` | İşaretli sıfır bileşenleri | Her bileşenin sıfır işareti |
| `subnormal-and-large` | Subnormal ve en büyük sonlu bileşenler | Sınır bileşen bitleri |
| `infinities` | Sonsuz ve sonlu bileşenler | Sonsuzluk işaretlerinin çevrilmesi |

Girdiler `family: "math"`, `operation: "vec3"` ve `steps` içinde `reverse` kullanır. Vec3 bileşenleri 16 haneli küçük harfli f64 bit string'leridir; `result.events` başlangıç ve her adım sonrası snapshot'ı taşır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::reverse`, Java'nın `scale(-1.0)` yönlendirmesini üretim API'sine taşır. `examples/behavior_adapter.rs` çağrıyı JSONL adımına bağlar.
