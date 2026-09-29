# P01.b — Vec3 relative sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.relative(Direction, double)` metodunun yön normalini ölçekleyip mevcut Vec3'e eklemesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:299-302` içindeki `relative` metodu ile `core/Direction.java:253-265,383-385` içindeki yön normalinin alınması incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `Direction` normalinin X, Y ve Z adımları sırasıyla `-1`, `0` veya `1` değerleridir; altı yönün her biri ayrı ayrı korunur.
- Her çıktı bileşeni Java'daki `this.component + distance * normal.component` işlem sırasıyla hesaplanır.
- Çarpan ve bileşen değerleri normalleştirilmez; signed zero, subnormal, sonlu sınır, Infinity, overflow ve NaN bitleri Java sonucuyla karşılaştırılır.
- Sonuç yeni bir Vec3'tür; başlangıç bileşenleri değiştirilmez ve snapshot bileşenleri toleranssız f64 bitleriyle taşınır.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `east-basic` | Sonlu bileşenler, `direction: east`, pozitif sonlu uzaklık | Pozitif X adımı |
| `west-signed-zero` | Signed zero, subnormal bileşen ve `distance: +0.0`, `direction: west` | Negatif X adımı ve sıfır işareti |
| `up-subnormal` | Negatif subnormal Y, en büyük sonlu Z ve en küçük pozitif subnormal uzaklık | Pozitif Y adımı ve subnormal toplama |
| `down-overflow` | Negatif Infinity Y ve en büyük sonlu uzaklık, `direction: down` | Negatif Y adımı ve sonlu sınır çarpımı |
| `north-cancellation` | En büyük sonlu X/Z ve negatif subnormal Y, en büyük sonlu uzaklık | Negatif Z adımı ve sınırda iptal |
| `south-special` | Sonlu X/Y, pozitif Infinity Z ve payload taşıyan NaN uzaklık | Pozitif Z adımı ve NaN yayılımı |

Girdiler `family: "math"`, `operation: "vec3_relative"`, `first` Vec3 nesnesi, `direction` adı ve 16 haneli f64 bit string'i `distance` alanından oluşur. Sonuç `value` içinde Vec3 snapshot'ı olarak döner.

## Rust karşılığı

`crates/voxel-core/src/math.rs` içindeki `Vec3::relative`, `Direction::step()` değerlerini f64 uzaklıkla çarpıp Java bileşen sırasıyla toplar. `examples/behavior_adapter.rs` yönü `Direction::from_name` ile çözer ve uzaklığı ham bitlerden okur; `tests/behavior/java/MathOracle.java` aynı `Vec3.relative` çağrısını doğrudan oracle çıktısına bağlar.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Vec3.relative(Direction, double)` metodunu karşılar. Rust adaptörü yön normalini uzaklıkla birleştirir; Java oracle doğrudan `Vec3.relative` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Altı yeni senaryoda EAST/WEST, UP/DOWN ve NORTH/SOUTH yönleri; signed zero, subnormal, sonlu sınır, Infinity, overflow ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.relative(Direction, double)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Direction.getUnitVec3i` normalinin diğer tüketicileri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
