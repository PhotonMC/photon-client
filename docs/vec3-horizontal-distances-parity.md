# P01.b — Vec3 horizontalDistance doğrulaması

`src/math.rs`, Java `Vec3.horizontalDistance()` ve `Vec3.horizontalDistanceSqr()` metodlarının yalnızca x/z karelerini, kaynak işlem sırasını ve karekök adımını karşılar. Java oracle iki metodu doğrudan çağırır; Rust adaptörü aynı girdideki iki Rust API sonucunu taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 893 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **893/893** math senaryosunda eşleşti.
- Beş yeni senaryoda temel x/z hesabı, kaynak toplama sırası, signed zero, subnormal underflow, Infinity ve kullanılmayan y bileşeni karşılaştırıldı.
- Birleşik altı aile corpus'u **1347/1347** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1347 senaryoluk Java referansını yeniden doğruladı.

Math referansı `14e060de3e2dfd133faf305681288b099fffc157d99d2140efcced661186142a`, birleşik referans `2ab36c787d438222d85bab3a2d9318b031512b21c923892daf5b2c0ad183f52c` kimliğindedir.

Yerel raporlar: `compare-gnxkpf0j` (math debug), `compare-u696_8k6` (math release), `compare-xswxfvez` (birleşik debug), `compare-n8easogi` (birleşik release) ve `verify-y0ggqu0q` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün Vec3 girdisinde iki yatay uzaklık metodunun sonuçlarıyla sınırlıdır. `length`, `lengthSqr`, `horizontal`, `offsetRandom` ve diğer Vec3 yardımcıları ile overload'lar ayrı dilimlerdir.
