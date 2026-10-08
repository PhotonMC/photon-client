# P01.b — Vec3 rotateClockwise90 sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.rotateClockwise90()` metodunun saat yönündeki 90 derece eksen permütasyonunu içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:259-261` içindeki metod incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Sonuç bileşenleri `(−z, y, x)` sırasıyla yeni bir Vec3 içinde oluşturulur.
- `z` bileşeninin unary negation işlemi yalnızca işaret bitini çevirir; y ve x ham f64 bitleriyle taşınır.
- Metod trigonometrik hesap, clamp veya mutasyon yapmaz; Infinity, NaN, subnormal ve signed zero değerleri aynı IEEE-754 işlemiyle gözlenir.
- Başlangıç ve sonuç Vec3 snapshot'ları toleranssız ham f64 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Sonlu pozitif/negatif bileşenler | `(−z, y, x)` permütasyonu |
| `negative-components` | Farklı işaretli sonlu bileşenler | İşaret ve eksen sırası |
| `signed-zero` | Signed-zero bileşenler | Unary negation ile sıfır işareti |
| `subnormal-and-large` | En küçük subnormal ve en büyük sonlu değer | Sınır değerleri ve işaret çevirme |
| `special-values` | Infinity ve NaN bileşenleri | Özel değerlerin doğrudan taşınması |

Girdiler `family: "math"`, `operation: "vec3_rotate_clockwise_90"` ve on altı haneli f64 bit string'lerinden oluşan `first` Vec3 alanını taşır. Sonuç `value` alanında Vec3 snapshot'ıdır; bileşenler, uzunluk ve uzunluk karesi on altı haneli küçük harfli f64 bit string'leridir.

## Rust karşılığı

`crates/photon-core/src/math.rs` içindeki `Vec3::rotate_clockwise_90`, Java'nın `-z`, `y`, `x` sırasını ve unary negation işaret bitini korur. `examples/behavior_adapter.rs` snapshot döndürür; `tests/behavior/java/MathOracle.java` aynı `rotateClockwise90` çağrısını doğrudan oracle çıktısına bağlar.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Vec3.rotateClockwise90()` metodunu karşılar. Rust adaptörü sonucu `(-z, y, x)` bileşenleriyle üretir; Java oracle doğrudan `Vec3.rotateClockwise90` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda eksen permütasyonu, negatif bileşenler, signed zero, subnormal/en büyük sonlu değerler ve Infinity/NaN taşınması karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.rotateClockwise90` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. Trigonometrik rotasyonlar, yön dönüşümleri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
