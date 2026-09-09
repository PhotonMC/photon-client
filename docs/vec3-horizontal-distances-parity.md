# P01.b — Vec3 horizontalDistance doğrulaması

`src/math.rs`, Java `Vec3.horizontalDistance()` ve `Vec3.horizontalDistanceSqr()` metodlarının yalnızca x/z karelerini, kaynak işlem sırasını ve karekök adımını karşılar. Java oracle iki metodu doğrudan çağırır; Rust adaptörü aynı girdideki iki Rust API sonucunu taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 813 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **813/813** math senaryosunda eşleşti.
- Beş yeni senaryoda temel x/z hesabı, kaynak toplama sırası, signed zero, subnormal underflow, Infinity ve kullanılmayan y bileşeni karşılaştırıldı.
- Birleşik altı aile corpus'u **1267/1267** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1267 senaryoluk Java referansını yeniden doğruladı.

Math referansı `28f7cb5106f4afaea9ccbb7f1a5601e99e2bf46d20a240f2d4288eb8fa0810ca`, birleşik referans `6b1e215c94a31ad5e80b7ab921054436873666bf0c583fb7b768cc855632bde2` kimliğindedir.

Yerel raporlar: `compare-nv2dy1tn` (math debug), `compare-zi0twrtb` (math release), `compare-lny6p0bp` (birleşik debug), `compare-g4_lvtxx` (birleşik release) ve `verify-nf8ea9dc` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün Vec3 girdisinde iki yatay uzaklık metodunun sonuçlarıyla sınırlıdır. `length`, `lengthSqr`, `horizontal`, `offsetRandom` ve diğer Vec3 yardımcıları ile overload'lar ayrı dilimlerdir.
