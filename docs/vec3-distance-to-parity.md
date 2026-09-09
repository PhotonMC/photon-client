# P01.b — Vec3 distanceTo doğrulaması

`src/math.rs`, Java `Vec3.distanceTo(Vec3)` metodunun hedef-minus-mevcut fark sırasını, x/y/z karelerinin kaynak sırasındaki toplamını ve son `sqrt` çağrısını karşılar. Java oracle doğrudan `distanceTo` çağırır; Rust adaptörü sonucu f64 bitleriyle JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 973 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **973/973** math senaryosunda eşleşti.
- Beş yeni senaryoda temel uzaklık, kaynak toplama sırası, signed zero, subnormal underflow ve overflow/özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1475/1475** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1475 senaryoluk Java referansını yeniden doğruladı.

Math referansı `5b9ac04dfbc4d0f113189cc9a302a301459740911a59e19bb68a172015f61780`, birleşik referans `6be1cb26871cc94c222742a0c5ccb50c6ef5df50a6b4f33206aeefdd805d2a60` kimliğindedir.

Yerel raporlar: `compare-yzj8g47` (math debug), `compare-0gyul_vz` (math release), `compare-1qxbdgol` (birleşik debug), `compare-9okhsl5l` (birleşik release) ve `verify-v5w4qfr` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 uzaklık sorgusuyla sınırlıdır. `distanceToSqr`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
