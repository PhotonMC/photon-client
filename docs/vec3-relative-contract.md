# P01.b — Vec3 relative sözleşmesi

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

`src/math.rs` içindeki `Vec3::relative`, `Direction::step()` değerlerini f64 uzaklıkla çarpıp Java bileşen sırasıyla toplar. `examples/behavior_adapter.rs` yönü `Direction::from_name` ile çözer ve uzaklığı ham bitlerden okur; `tests/behavior/java/MathOracle.java` aynı `Vec3.relative` çağrısını doğrudan oracle çıktısına bağlar.
