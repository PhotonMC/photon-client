# P01.b — Vec3 distanceTo doğrulaması

`src/math.rs`, Java `Vec3.distanceTo(Vec3)` metodunun hedef-minus-mevcut fark sırasını, x/y/z karelerinin kaynak sırasındaki toplamını ve son `sqrt` çağrısını karşılar. Java oracle doğrudan `distanceTo` çağırır; Rust adaptörü sonucu f64 bitleriyle JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 695 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **695/695** math senaryosunda eşleşti.
- Beş yeni senaryoda temel uzaklık, kaynak toplama sırası, signed zero, subnormal underflow ve overflow/özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1149/1149** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1149 senaryoluk Java referansını yeniden doğruladı.

Math referansı `54c59289cd211aca498d2ecb24a011580a24adfcfc88e7993e2fe339251990a7`, birleşik referans `5f4fd0e48a09b99596c7d025768d8535f8c5ee702c6ef4dffd614eafae39c7e5` kimliğindedir.

Yerel raporlar: `compare-z24w3ncm` (math debug), `compare-ue0l25nl` (math release), `compare-oiqblje5` (birleşik debug), `compare-pl0oao3t` (birleşik release) ve `verify-kyxqm2yj` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 uzaklık sorgusuyla sınırlıdır. `distanceToSqr`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
