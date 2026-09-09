# P01.b — Vec3 vectorTo doğrulaması

`src/math.rs`, Java `Vec3.vectorTo(Vec3)` metodunu karşılayan `Vec3::vector_to` metodunu içerir. Java oracle doğrudan `vectorTo` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 670 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **670/670** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu fark, ardışık hedef değişimi, signed zero, double taşması, Infinity ve NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1124/1124** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1124 senaryoluk Java referansını yeniden doğruladı.

Math referansı `233afc7fe403783bb0e269123225910816ff5ec1075c4bf10cdf8bbbcdc2853f`, birleşik referans `7463d45cd07a5207a5d29e6be51ea489c541efadb966932e595b98fdda8d1940` kimliğindedir.

Yerel raporlar: `compare-ra03eisz` (math debug), `compare-up24hdi8` (math release), `compare-rgbylm8r` (birleşik debug), `compare-445klgjk` (birleşik release) ve `verify-bt555rfc` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distance` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
