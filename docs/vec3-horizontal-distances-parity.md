# P01.b — Vec3 horizontalDistance doğrulaması

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
