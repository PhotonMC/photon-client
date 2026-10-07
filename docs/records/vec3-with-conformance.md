# P01.b — Vec3 with sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.with(Direction.Axis, double)` metodunun seçilen bileşeni yeni değerle değiştirip diğer bileşenleri korumasını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:292-297` içindeki `with` metodu ve `core/Direction.java:402-462` içindeki eksen değerleri incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `Axis.X`, yalnızca x bileşenini; `Axis.Y`, yalnızca y bileşenini; `Axis.Z`, yalnızca z bileşenini verilen f64 değerle değiştirir.
- Değiştirilmeyen bileşenlerin IEEE-754 bitleri aynen korunur.
- Yeni değer aritmetik veya normalizasyondan geçmez; signed zero, subnormal, sonlu sınır, Infinity ve NaN bitleri doğrudan taşınır.
- Sonuç yeni bir Vec3'tür; giriş vektörü değiştirilmez ve tüm bileşenler ham bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `x` | Sonlu bileşenler, `axis: x`, yeni sonlu değer | X bileşeni değişimi |
| `y-signed-zero` | Signed zero ve büyük sonlu değer, `axis: y`, negatif sıfır | Yeni signed zero bitinin korunması |
| `z-subnormal` | Sonlu bileşenler, `axis: z`, en küçük pozitif subnormal | Z bileşeni ve subnormal değer |
| `x-special-value` | Infinity, payload taşıyan NaN ve negatif Infinity, `axis: x`, NaN payload | Özel yeni değerin ham bitleri |
| `y-max-finite` | Subnormal ve signed zero bileşenleri, `axis: y`, en büyük sonlu değer | Korunan bileşen bitleri ve sonlu sınır |

Girdiler `family: "math"`, `operation: "vec3_with"`, `first` Vec3 nesnesi, `axis` adı ve 16 haneli f64 bit string'i `value` alanından oluşur. Sonuç `value` içinde Vec3 snapshot'ı olarak döner.

## Rust karşılığı

`crates/voxel-core/src/math.rs` içindeki `Vec3::with`, seçilen `Axis` için yalnızca ilgili bileşeni değiştirerek yeni Vec3 üretir. `examples/behavior_adapter.rs` ekseni `Axis::from_name` ile çözer ve f64 değerini ham bitlerden okur; `tests/behavior/java/MathOracle.java` aynı `Vec3.with` çağrısını doğrudan oracle çıktısına bağlar.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Vec3.with(Direction.Axis, double)` metodunu karşılar. Rust adaptörü seçilen bileşeni verilen f64 değere taşır ve diğer bileşen bitlerini korur; Java oracle doğrudan `Vec3.with` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda X/Y/Z seçimi, değiştirilmemiş bileşenlerin korunması, signed zero, subnormal, en büyük sonlu değer, Infinity ve NaN payload bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.with(Direction.Axis, double)` sonucunun ham f64 snapshot bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
