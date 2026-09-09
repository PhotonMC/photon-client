# P01.b — Vec3 horizontal sözleşmesi

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.horizontal()` metodudur.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:168-170` içindeki `horizontal` gövdesi incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Yeni Vec3 mevcut x ve z bileşenlerini aynı bit değerleriyle taşır.
- Yeni vektörün y bileşeni sabit pozitif `0.0` olur; mevcut y bileşeni okunur fakat sonuca taşınmaz.
- Girdi vektörü değişmez ve her adımın sonucu sonraki adımın girdisi olur.
- Signed zero, subnormal, Infinity ve taşma sonuçları ham IEEE-754 f64 bitleriyle karşılaştırılır; tolerans uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu üç bileşen | x/z korunması ve y sıfırlaması |
| `chain` | Horizontal sonrası bileşen işlemi | Sıralı snapshot'lar |
| `signed-zero` | İşaretli sıfır bileşenleri | x/z işaretleri ve y pozitif sıfırı |
| `subnormal-and-large` | Subnormal x/z ve en büyük sonlu y | Sınır bileşen bitleri |
| `infinities` | Sonsuz x/y/z | x/z sonsuzlukları ve y sıfırı |

Girdiler `family: "math"`, `operation: "vec3"` ve `steps` içinde `horizontal` kullanır. Vec3 bileşenleri 16 haneli küçük harfli f64 bit string'leridir; `result.events` başlangıç ve her adım sonrası snapshot'ı taşır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::horizontal`, Java'nın x/z kopyası ve pozitif sıfır y bileşeni davranışını üretim API'sine taşır. `examples/behavior_adapter.rs` çağrıyı JSONL adımına bağlar.
