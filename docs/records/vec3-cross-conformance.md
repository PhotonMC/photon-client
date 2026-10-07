# P01.b — Vec3 cross sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `Vec3.cross(Vec3)` metodunun yeni bir vektör üretmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:91-93` içindeki `cross(Vec3)` incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `cross(Vec3)`, iki Vec3'ün üç determinant bileşenini üretir ve sonucu yeni bir Vec3 olarak döndürür.
- Her determinant bileşeni iki çarpımın çıkarılmasıyla hesaplanır; x, y ve z bileşenleri kaynakta görülen sırayı korur.
- Girdi vektörleri değişmez; ardışık step çağrılarında bir önceki sonuç sonraki `cross` çağrısının ilk operandıdır.
- Signed zero, NaN, Infinity ve double taşması sonuçları ham IEEE-754 bitleriyle karşılaştırılır; tolerans veya NaN normalizasyonu uygulanmaz.
- Sonuç snapshot'ı her adımda bileşen, uzunluk ve uzunluk karesi bitlerini taşır; türetilmiş ölçümlerdeki taşma ve özel değerler de kapsam içindedir.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu, farklı bileşenli iki Vec3 | Cross vektörü ve türetilmiş ölçümler |
| `chain` | İki cross ve sonraki normalize adımı | Sıralı vektör snapshot'ları |
| `signed-zero` | İşaretli sıfır bileşenleri | Bileşen bitleri |
| `overflow` | En büyük sonlu double ile çarpım | Infinity ve vektör bitleri |
| `special-values` | Infinity ve sıfır çarpımı | NaN/Infinity bitleri |

Girdiler `family: "math"`, `operation: "vec3"` ve `steps` içinde `cross` kullanır. İkinci vektör `other` nesnesinde, bileşenler 16 haneli küçük harfli f64 bit string'leri olarak taşınır.

## Rust karşılığı

`crates/voxel-core/src/math.rs` içindeki `Vec3::cross`, üç determinant bileşenini doğrudan kaynak operand sırasıyla hesaplar. `examples/behavior_adapter.rs` bu metodu JSONL `cross` adımına bağlar; `tests/behavior/java/MathOracle.java` beklenen sonucu doğrudan Java `cross` çağrısından alır.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Vec3.cross(Vec3)` metodunu karşılayan `Vec3::cross` metodunu içerir. Java oracle doğrudan `cross` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda temel determinant, ardışık cross, signed zero, double overflow ve NaN/Infinity sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve her adımın vektör snapshot'larıyla sınırlıdır. `dot`, `distanceToSqr`, `distanceTo`, closerThan ve diğer Vec3 yardımcıları ayrı dilimlerdir.
