# P01.b — Vec3 toString sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, `Vec3.toString()` sonucunun üç double bileşeni ve sabit ayraçlarla biçimlendirmesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:224-226` içindeki `toString` gövdesi incelendi. Oracle doğrudan Java metodunu çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Sonuç `(x, y, z)` biçimindedir; parantez, virgül ve boşluklar sabittir.
- Her bileşen Java `Double.toString` kurallarıyla yazılır. Ondalık/bilimsel gösterim eşiği, `.0`, signed zero, `Infinity`, `-Infinity` ve `NaN` metinleri gözlenebilir.
- `toString()` vektörün bileşenlerini değiştirmez.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `basic` | Sonlu ve asimetrik bileşenler | Sabit metin biçimi ve temel double yazımı |
| `scientific` | Küçük/büyük değerler ve signed zero | Bilimsel gösterim ve işaretli sıfır |
| `thresholds` | `1e-3`, `1e7` komşuları | Java double gösterim eşikleri |
| `signed-zero` | Pozitif ve negatif sıfır | Sıfır işaretinin korunması |
| `infinity` | Pozitif/negatif Infinity | Sonsuzluk metinleri |
| `nan` | Farklı NaN bit desenleri | NaN metni ve payload bağımsızlığı |

Girdiler `family: "math"`, `operation: "vec3_to_string"` ve f64 bit dizgeli `first` Vec3 nesnesinden oluşur. Sonuç, `value` alanında Java metni olarak taşınır.

## Rust karşılığı

`crates/photon-core/src/math.rs` içindeki `Vec3::to_string_java`, Java bileşen biçimini ortak `java_double_to_string` yordamıyla üretir. Aynı yordam AABB ve VoxelShape metinlerinde de kullanılır; `examples/behavior_adapter.rs` sonucu taşır, `tests/behavior/java/MathOracle.java` Java `Vec3.toString()` metodunu çağırır.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Vec3.toString()` davranışını `Vec3::to_string_java` ile karşılar. Java oracle, sonucu doğrudan Java metodundan alır; double biçimlendirme yordamı AABB metin yolu ile paylaşılır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Altı yeni senaryoda temel biçim, bilimsel gösterim eşikleri, signed zero, Infinity ve farklı NaN bit desenleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 metninin Java çıktısına byte düzeyinde eşitliğiyle sınırlıdır. Java `Double.toString` için daha geniş tüm bit desenleri matrisi ve diğer sınıfların metin biçimleri ayrı dilimlerdir.
