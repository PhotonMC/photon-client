# P01.b — Vec3 horizontalDistance sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **5 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.horizontalDistance()` ve `Vec3.horizontalDistanceSqr()` metodlarını birlikte içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:188-196` içindeki iki gövde incelendi. Oracle doğrudan Java çağrılarını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `horizontalDistanceSqr()` x ve z bileşenlerini ayrı ayrı karesine alır ve `x * x + z * z` kaynak sırasıyla toplar.
- `horizontalDistance()` aynı x/z toplamının `Math.sqrt` sonucunu döndürür.
- y bileşeni iki metodun hesabına girmez; girdi Vec3 değişmez.
- Signed zero, subnormal, büyük sonlu ve Infinity sonuçları ham IEEE-754 f64 bitleriyle karşılaştırılır; tolerans uygulanmaz.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu x/y/z bileşenleri | Yatay uzaklık ve karesi |
| `order-and-rounding` | Büyük x ve küçük z karesi | x/z toplama sırası |
| `signed-zero` | İşaretli sıfır x/z bileşenleri | Uzaklık ve kareli uzaklık sıfır bitleri |
| `subnormal` | En küçük subnormal x/z bileşenleri | Kare alma underflow sonucu |
| `infinities` | Sonsuz x/z ve kullanılmayan NaN y | Sonsuzluk sonuçları; y'nin dışlanması |

Girdiler `family: "math"`, `operation: "vec3_horizontal_distances"` ve `first` Vec3 nesnesini taşır. Sonuç `distance` ve `distance_squared` alanlarında 16 haneli küçük harfli f64 bit string'leri bulunur.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::horizontal_distance` ve `Vec3::horizontal_distance_squared`, Java'nın x/z işlem sırasını ve karekök adımını taşır. `examples/behavior_adapter.rs` JSONL operasyonunu, `tests/behavior/java/MathOracle.java` ise iki beklenen değeri doğrudan Java çağrılarına bağlar.


## Client-parity doğrulaması

`src/math.rs`, Java `Vec3.horizontalDistance()` ve `Vec3.horizontalDistanceSqr()` metodlarının yalnızca x/z karelerini, kaynak işlem sırasını ve karekök adımını karşılar. Java oracle iki metodu doğrudan çağırır; Rust adaptörü aynı girdideki iki Rust API sonucunu taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Beş yeni senaryoda temel x/z hesabı, kaynak toplama sırası, signed zero, subnormal underflow, Infinity ve kullanılmayan y bileşeni karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün Vec3 girdisinde iki yatay uzaklık metodunun sonuçlarıyla sınırlıdır. `length`, `lengthSqr`, `horizontal`, `offsetRandom` ve diğer Vec3 yardımcıları ile overload'lar ayrı dilimlerdir.
