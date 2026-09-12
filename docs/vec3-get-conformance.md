# P01.b — Vec3 get sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.get(Direction.Axis)` metodunun seçilen eksenin ham f64 bileşenini döndürmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:288` ve `core/Direction.java:402-462,541` içindeki `get` ve `Axis.choose(double, double, double)` yolları incelendi. Oracle doğrudan Java çağrısını kullanır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `Axis.X`, `x`; `Axis.Y`, `y`; `Axis.Z`, `z` bileşenini doğrudan seçer.
- Seçim aritmetik, normalizasyon veya dönüşüm yapmaz; seçilen değerin IEEE-754 f64 bitleri korunur.
- Signed zero, subnormal, sonlu sınır, Infinity ve NaN seçilebilir değerlerdir ve ham bitleriyle karşılaştırılır.
- Vektör değiştirilmez; sonuç toleranssız 16 haneli f64 bit string'i olarak taşınır.

## Senaryolar ve JSONL

| Senaryo | Girdi / sınır | Kapsanan davranış |
| --- | --- | --- |
| `x` | Sonlu ve işaretli bileşenler, `axis: x` | X bileşeni seçimi |
| `y` | Negatif ve pozitif sonlu bileşenler, `axis: y` | Y bileşeni seçimi |
| `z-signed-zero` | Pozitif ve negatif sıfırlar, `axis: z` | Signed zero bitinin korunması |
| `subnormal-and-large` | Subnormal, negatif subnormal ve en büyük sonlu değer, `axis: z` | Sınır değerlerin aritmetiksiz seçimi |
| `special-values` | Infinity, payload taşıyan NaN ve negatif Infinity, `axis: y` | Özel f64 bitlerinin korunması |

Girdiler `family: "math"`, `operation: "vec3_get"`, `first` Vec3 nesnesi ve `axis` olarak taşınır. Bileşenler 16 haneli küçük harfli f64 bit string'leridir; sonuç `value` alanında aynı biçimde döner.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::get`, `Axis` eşleşmesiyle x/y/z bileşenini doğrudan döndürür. `examples/behavior_adapter.rs` ekseni mevcut `Axis::from_name` yoluyla çözer ve sonucu bit string olarak taşır; `tests/behavior/java/MathOracle.java` aynı `Vec3.get` çağrısını doğrudan oracle çıktısına bağlar.


## Client-parity doğrulaması

`src/math.rs`, Java `Vec3.get(Direction.Axis)` metodunu karşılar. Rust adaptörü seçilen eksenin bileşen bitlerini döndürür; Java oracle doğrudan `Vec3.get` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda üç eksen, signed zero, subnormal, en büyük sonlu değer, Infinity ve payload taşıyan NaN ham bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Math karşılaştırma raporları `compare-00tgiqc4` (debug) ve `compare-aylr4ozm` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release), Java doğrulaması `verify-mgwg6r2l` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.get(Direction.Axis)` sonucunun ham f64 bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
