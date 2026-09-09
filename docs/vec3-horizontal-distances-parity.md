# P01.b — Vec3 horizontalDistance doğrulaması

`src/math.rs`, Java `Vec3.horizontalDistance()` ve `Vec3.horizontalDistanceSqr()` metodlarının yalnızca x/z karelerini, kaynak işlem sırasını ve karekök adımını karşılar. Java oracle iki metodu doğrudan çağırır; Rust adaptörü aynı girdideki iki Rust API sonucunu taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 725 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **725/725** math senaryosunda eşleşti.
- Beş yeni senaryoda temel x/z hesabı, kaynak toplama sırası, signed zero, subnormal underflow, Infinity ve kullanılmayan y bileşeni karşılaştırıldı.
- Birleşik altı aile corpus'u **1179/1179** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1179 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ba2c0a50dc0b1df009707b281c6734a888c59ecbfee1929dc86125f395920582`, birleşik referans `3be9baa80520a67f762a0d1221d9e4a40892597f1dc8ccba80afd18b3fe8689b` kimliğindedir.

Yerel raporlar: `compare-7nv0ftym` (math debug), `compare-omc6avwk` (math release), `compare-cogkg_y3` (birleşik debug), `compare-vj0crhe_` (birleşik release) ve `verify-kco3t37r` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün Vec3 girdisinde iki yatay uzaklık metodunun sonuçlarıyla sınırlıdır. `length`, `lengthSqr`, `horizontal`, `offsetRandom` ve diğer Vec3 yardımcıları ile overload'lar ayrı dilimlerdir.
