# P01.d — AABB.Builder sözleşmesi

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-builder-parity.md)). Kapsam, `AABB.Builder` sınıfının f32 noktaları biriktirmesini, tanımlı durumunu ve kutu üretimindeki hata yolunu içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:469-499` incelendi. `MathOracle`, `AABB.Builder` ve JOML `Vector3f` çağrılarını doğrudan çalıştırır; beklenen kutu Rust çıktısından üretilmez.

## Davranış sözleşmesi

- Yeni builder'ın alt sınırları f32 `+Infinity`, üst sınırları f32 `-Infinity` ve `isDefined()` sonucu `false` olur.
- `include(Vector3fc)`, her eksende `Math.min` ve `Math.max` ile f32 koordinatları biriktirir; tek bir nokta dahil edildiğinde `defined` bayrağı `true` olur.
- Biriktirme signed zero ve NaN kurallarını Java f32 `Math.min/max` ile korur. Noktanın f32 değeri build sırasında double'a genişletilir.
- Tanımsız builder'da `build()` `IllegalStateException` üretir. Rust karşılığı `Err(AabbBuilderError::Undefined)` değeridir; hata mesajı Java'daki “Cannot build an undefined AABB. Include at least one point.” metnidir.
- Tanımlı builder'da `build()` bir AABB constructor'ı çağırır ve builder durumunu değiştirmez. Rust API'si `include(x, y, z)` ile `Vector3fc` bileşenlerini f32 olarak alır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `empty` | Nokta yok | Başlangıç `defined` durumu ve tanımsız build hatası |
| `single` | Tek f32 nokta | Noktanın alt/üst endpoint'leri |
| `multiple` | Sırası karışık üç nokta | Her eksenin min/max birikimi |
| `float-precision` | Ondalıklı f32 noktalar | F32 yuvarlamasının double endpoint'e taşınması |
| `special-values` | Signed zero, NaN ve sonsuzluk | Java özel değer ve signed zero kuralları |

Girdiler `family: "math"`, `operation: "aabb_builder"` ve f32 bit string'lerinden oluşan `points` alanını kullanır. `events`, başlangıç ve her `include` sonrasındaki `isDefined()` değerlerini taşır; `build`, değer veya `undefined` hatasıdır.

## Rust karşılığı

`src/aabb.rs` içindeki `AabbBuilder`, Java sentinellerini, f32 min/max yardımcılarını ve double'a genişleyen `build` yolunu uygular. `examples/behavior_adapter.rs`, JSONL işlemini üretim API'sine yönlendirir.
