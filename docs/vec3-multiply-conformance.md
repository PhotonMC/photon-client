# P01.b — Vec3 multiply overload sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `Vec3.multiply` overload'larıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:160-166` içindeki `multiply(Vec3)` ve `multiply(double, double, double)` gövdeleri incelendi. Oracle doğrudan Java overload çağrılarını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `multiply(Vec3)`, ölçek vektörünün x, y ve z bileşenlerini üç bileşenli overload'a kaynak sırasıyla aktarır.
- Üç bileşenli overload mevcut vektörün x, y ve z bileşenlerini karşılık gelen ölçeklerle çarpar ve yeni bir Vec3 üretir.
- Girdi vektörü değiştirilmez; her adımın sonucu sonraki adımın girdisi olur.
- Signed zero, subnormal, NaN, Infinity ve taşma sonuçları ham IEEE-754 f64 bitleriyle karşılaştırılır; tolerans veya NaN normalizasyonu uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vector-basic` | Vec3 ölçek operand'ı | `multiply(Vec3)` sonrası snapshot |
| `components-basic` | Ayrı x/y/z ölçekleri | Üç bileşenli overload sonrası snapshot |
| `overload-chain` | İki overload'ın ardışık kullanımı | Kaynak işlem sırasındaki snapshot'lar |
| `signed-zero` | İşaretli sıfır bileşenleri ve ölçekleri | Bileşen bitleri |
| `overflow-and-special-values` | En büyük sonlu değer, Infinity ve NaN | Taşma/özel değer bitleri |

Girdiler `family: "math"`, `operation: "vec3"` ve `steps` içinde `multiply_vec3` veya `multiply_components` kullanır. Vec3 bileşenleri ve ölçekler 16 haneli küçük harfli f64 bit string'leridir; `result.events` başlangıç ve her adım sonrası snapshot'ı taşır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::multiply_vec3` ve `Vec3::multiply_components`, Java overload yönlendirmesini ve x/y/z çarpım sırasını üretim API'sine taşır. `examples/behavior_adapter.rs` bu çağrıları JSONL adımlarına, `tests/behavior/java/MathOracle.java` ise doğrudan Java çağrılarına bağlar.


## Client-parity doğrulaması

`src/math.rs`, Java `Vec3.multiply(Vec3)` ve `Vec3.multiply(double, double, double)` overload'larının bileşen yönlendirmesini ve x/y/z çarpımlarını karşılar. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, overload zinciri, signed zero, taşma ve NaN/Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `scale`, `reverse`, `horizontal`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
