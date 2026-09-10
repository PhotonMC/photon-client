# P01.b — Vec3 add/subtract overload doğrulaması

`src/math.rs`, Java `Vec3.add` ve `Vec3.subtract` overload'larını karşılayan altı Rust metodu içerir. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1478 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1478/1478** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3, scalar ve üç bileşenli overload zincirleri; signed zero, Infinity ve NaN girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1980/1980** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1980 senaryoluk Java referansını yeniden doğruladı.

Math referansı `abe3d8cdf014db1ee9b2fa953ff1b7ba7bc40e31c9d35672c0cd8adb20896f50`, birleşik referans `e068e38cc1d0ab64ac2844f20d474f313e2325b42cc620af39051ca8aa6748cd` kimliğindedir.

Yerel raporlar: `compare-xrmyz06w` (math debug), `compare-zrxwzjp_` (math release), `compare-dxdmken0` (birleşik debug), `compare-cvif6gdr` (birleşik release) ve `verify-jlkp_8wf` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
