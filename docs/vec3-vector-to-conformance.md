# P01.b — Vec3 vectorTo sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `Vec3.vectorTo` metodunun mevcut vektörden hedef vektöre yönlü fark üretmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:79-81` içindeki `vectorTo(Vec3)` incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `vectorTo(Vec3)` hedef vektörün her bileşeninden mevcut vektörün aynı bileşenini çıkarır ve yeni bir `Vec3` üretir.
- Çıkarma operand sırası hedef eksi mevcut şeklindedir; sonuç bileşenleri f64 aritmetiğinin Java'daki bit sonucunu taşır.
- Mevcut ve hedef vektörler değişmez; ardışık çağrılarda önceki sonuç sonraki çağrının mevcut vektörüdür.
- Signed zero, NaN, Infinity ve taşma sonuçları için tolerans veya NaN normalizasyonu uygulanmaz.
- Her senaryo başlangıç ve her adım sonrası `x`, `y`, `z`, `length`, `length_squared` bitlerini kaydeder.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu mevcut ve hedef vektörler | Fark vektörü ve türetilmiş ölçümler |
| `chain` | İki vectorTo ve sonraki add/normalize adımları | Sıralı vektör snapshot'ları |
| `signed-zero` | İşaretli sıfır mevcut/hedef bileşenleri | Bileşen bitleri |
| `overflow` | En büyük sonlu double ve ters işaretli hedefler | Infinity ve vektör bitleri |
| `special-values` | Infinity ve NaN hedef bileşenleri | Özel değer bitleri |

Girdiler `family: "math"`, `operation: "vec3"` ve `steps` içinde `vector_to` kullanır. Hedef vektör `target` nesnesinde, bileşenler 16 haneli küçük harfli f64 bit string'leri olarak taşınır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::vector_to`, hedef bileşenlerinden mevcut bileşenleri doğrudan çıkarır. `examples/behavior_adapter.rs` bu metodu JSONL `vector_to` adımına bağlar; `tests/behavior/java/MathOracle.java` beklenen sonucu doğrudan Java `vectorTo` çağrısından alır.


## Client-parity doğrulaması

`src/math.rs`, Java `Vec3.vectorTo(Vec3)` metodunu karşılayan `Vec3::vector_to` metodunu içerir. Java oracle doğrudan `vectorTo` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu fark, ardışık hedef değişimi, signed zero, double taşması, Infinity ve NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
