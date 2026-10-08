# P01.b — Vec3 dot sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `Vec3.dot(Vec3)` metodunun üç bileşen çarpımını sıralı double toplamı olarak üretmesini içerir.

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

`crates/photon-core/src/math.rs` içindeki `Vec3::dot`, üç çarpımı ve toplamı doğrudan kaynak sırasıyla uygular. `examples/behavior_adapter.rs` `vec3_dot` operasyonunu, `tests/behavior/java/MathOracle.java` ise beklenen değeri doğrudan Java `dot` çağrısını kullanarak bağlar.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Vec3.dot(Vec3)` metodunu karşılayan `Vec3::dot` metodunu içerir. Java oracle doğrudan `dot` çağrısını yapar; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda temel çarpım, evaluation order/cancellation, signed zero, double overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 çifti ve dot sonuçlarının f64 bit snapshot'larıyla sınırlıdır. `cross`, `distanceToSqr`, `distanceTo`, closerThan ve diğer Vec3 yardımcıları ayrı dilimlerdir.
