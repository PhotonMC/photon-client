# P01.b — Vec3 reverse sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`src/math.rs`, Java `Vec3.reverse()` metodunun `scale(-1.0)` yönlendirmesini karşılar. Java oracle doğrudan çağrıyı yapar; Rust adaptörü aynı girdiyi `Vec3::reverse` API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu bileşenler, ardışık işlem, signed zero, subnormal, büyük sonlu ve Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve başlangıç/her adım snapshot'larıyla sınırlıdır. `scale`, `horizontal`, `offsetRandom`, `horizontalDistance`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
