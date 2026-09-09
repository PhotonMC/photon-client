# P01.b — Vec3 vectorTo doğrulaması

`src/math.rs`, Java `Vec3.vectorTo(Vec3)` metodunu karşılayan `Vec3::vector_to` metodunu içerir. Java oracle doğrudan `vectorTo` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 725 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **725/725** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu fark, ardışık hedef değişimi, signed zero, double taşması, Infinity ve NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1179/1179** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1179 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ba2c0a50dc0b1df009707b281c6734a888c59ecbfee1929dc86125f395920582`, birleşik referans `3be9baa80520a67f762a0d1221d9e4a40892597f1dc8ccba80afd18b3fe8689b` kimliğindedir.

Yerel raporlar: `compare-7nv0ftym` (math debug), `compare-omc6avwk` (math release), `compare-cogkg_y3` (birleşik debug), `compare-vj0crhe_` (birleşik release) ve `verify-kco3t37r` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
