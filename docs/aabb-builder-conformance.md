# P01.d — AABB.Builder sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `AABB.Builder` sınıfının f32 noktaları biriktirmesini, tanımlı durumunu ve kutu üretimindeki hata yolunu içerir.

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


## Client-parity doğrulaması

`src/aabb.rs`, Java `AABB.Builder` davranışını `AabbBuilder` ile uygular. `examples/behavior_adapter.rs`, f32 point bitlerini Rust API'sine taşır; Java oracle sonuçları doğrudan `AABB.Builder` çağrılarından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda boş builder, tekli ve çoklu nokta birikimi, f32 hassasiyeti ve signed zero/NaN/sonsuzluk yolları karşılaştırıldı.
- Tanımsız `build()` sonucu Java'daki `IllegalStateException` yoluna karşı `undefined` sonucu olarak doğrulandı.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik point dizisi ve bunların `isDefined()`/`build()` sonuçlarıyla sınırlıdır. `Vector3fc` için ayrı bir Rust vektör tipi eklenmedi; bileşenler f32 olarak alınır.
