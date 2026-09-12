# P01.b — Vec3 distanceTo sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `Vec3.distanceTo(Vec3)` metodunun tek overload'ıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:124-129` içindeki `distanceTo` gövdesi incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Hedef vektörün her bileşeninden mevcut vektör bileşeni çıkarılır: `xd = target.x - this.x` ve aynı işlem y/z için yapılır.
- Farklar ayrı ayrı karesine alınır, x, y, z terimleri kaynak sırasıyla toplanır ve toplamın `Math.sqrt` sonucu döndürülür.
- Girdi vektörleri değişmez ve sonuç tek bir f64 değerdir; kareli uzaklığa ayrı bir API çağrısı üzerinden yönlendirme sözleşmenin parçası değildir.
- Signed zero, subnormal, NaN, Infinity ve double taşması ham IEEE-754 bitleriyle karşılaştırılır; tolerans veya NaN normalizasyonu uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu mevcut ve hedef Vec3 | `distanceTo` sonucu |
| `order-and-rounding` | Büyük ve küçük fark kareleri | Kaynak toplama sırası ve karekök sonucu |
| `signed-zero` | İşaretli sıfır uçları | Pozitif sıfır sonucu |
| `subnormal` | En küçük subnormal farklar | Kare alma ve karekök underflow sonucu |
| `overflow-and-special-values` | En büyük sonlu double, Infinity ve NaN | Taşma/özel değer bitleri |

Girdiler `family: "math"` ve `operation: "vec3_distance_to"` kullanır. `first` ve `second` Vec3 nesnelerindeki tüm f64 bileşenleri ve sonuç `value` alanı 16 haneli küçük harfli bit string'leridir.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::distance_to`, fark değişkenlerini doğrudan kaynak sırasıyla hesaplar ve toplamı `sqrt` ile sonlandırır. `examples/behavior_adapter.rs` JSONL operasyonunu, `tests/behavior/java/MathOracle.java` ise beklenen değeri doğrudan Java overload çağrısına bağlar.


## Client-parity doğrulaması

`src/math.rs`, Java `Vec3.distanceTo(Vec3)` metodunun hedef-minus-mevcut fark sırasını, x/y/z karelerinin kaynak sırasındaki toplamını ve son `sqrt` çağrısını karşılar. Java oracle doğrudan `distanceTo` çağırır; Rust adaptörü sonucu f64 bitleriyle JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda temel uzaklık, kaynak toplama sırası, signed zero, subnormal underflow ve overflow/özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 uzaklık sorgusuyla sınırlıdır. `distanceToSqr`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
