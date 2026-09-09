# P01.b — Vec3 add/subtract overload doğrulaması

`src/math.rs`, Java `Vec3.add` ve `Vec3.subtract` overload'larını karşılayan altı Rust metodu içerir. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 725 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **725/725** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3, scalar ve üç bileşenli overload zincirleri; signed zero, Infinity ve NaN girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1179/1179** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1179 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ba2c0a50dc0b1df009707b281c6734a888c59ecbfee1929dc86125f395920582`, birleşik referans `3be9baa80520a67f762a0d1221d9e4a40892597f1dc8ccba80afd18b3fe8689b` kimliğindedir.

Yerel raporlar: `compare-7nv0ftym` (math debug), `compare-omc6avwk` (math release), `compare-cogkg_y3` (birleşik debug), `compare-vj0crhe_` (birleşik release) ve `verify-kco3t37r` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
