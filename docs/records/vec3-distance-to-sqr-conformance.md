# P01.b — Vec3 distanceToSqr sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `Vec3.distanceToSqr` metodunun Vec3 ve üç bileşenli overload'larını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:131-141` içindeki `distanceToSqr` overload'ları incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Vec3 overload'ı hedef vektörün her bileşeninden mevcut vektör bileşenini çıkarır; üç bileşenli overload aynı işlemi verilen hedef koordinatlarıyla yapar.
- Farklar ayrı ayrı karesine alınır ve x, y, z terimleri Java kaynak sırasıyla toplanır; karekök alınmaz.
- Girdi vektörü değişmez ve sonuç tek bir f64 değerdir.
- Signed zero, NaN, Infinity ve double taşması ham IEEE-754 bitleriyle karşılaştırılır; tolerans veya NaN normalizasyonu uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vector-basic` | Mevcut ve hedef Vec3 | Vec3 overload sonucu |
| `components-basic` | Mevcut Vec3 ve üç hedef bileşeni | Üç bileşenli overload sonucu |
| `order-and-rounding` | Büyük ve küçük fark kareleri | Kaynak toplama sırası sonucu |
| `signed-zero` | İşaretli sıfır farkları | Pozitif sıfır sonucu |
| `overflow-and-special-values` | En büyük sonlu double, Infinity ve NaN | Taşma/özel değer bitleri |

Girdiler `family: "math"` kullanır. Vec3 overload'ı `operation: "vec3_distance_to_sqr"` ile `first` ve `second` nesnelerini, üç bileşenli overload `operation: "vec3_distance_to_sqr_components"` ile `first`, `x`, `y`, `z` alanlarını taşır. Tüm f64 girdileri ve sonuç `value` alanı 16 haneli küçük harfli bit string'leridir.

## Rust karşılığı

`crates/photon-core/src/math.rs` içindeki `Vec3::distance_to_sqr` ve `distance_to_sqr_components`, fark karelerini doğrudan kaynak sırasıyla toplar. `examples/behavior_adapter.rs` iki JSONL operasyonunu, `tests/behavior/java/MathOracle.java` ise beklenen değerleri doğrudan Java overload çağrılarına bağlar.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Vec3.distanceToSqr(Vec3)` ve `Vec3.distanceToSqr(double, double, double)` overload'larını karşılayan iki Rust metodunu içerir. Java oracle doğrudan iki overload'u çağırır; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, kaynak toplama sırası, signed zero ve overflow/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 mesafe karesi sorgusuyla sınırlıdır. `distanceTo`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
