# P01.b — Vec3 rotateClockwise90 doğrulaması

`src/math.rs`, Java `Vec3.rotateClockwise90()` metodunu karşılar. Rust adaptörü sonucu `(-z, y, x)` bileşenleriyle üretir; Java oracle doğrudan `Vec3.rotateClockwise90` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 869 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **869/869** math senaryosunda eşleşti.
- Beş yeni senaryoda eksen permütasyonu, negatif bileşenler, signed zero, subnormal/en büyük sonlu değerler ve Infinity/NaN taşınması karşılaştırıldı.
- Birleşik altı aile corpus'u **1323/1323** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1323 senaryoluk Java referansını yeniden doğruladı.

Math referansı `6d26ec313384e0e1de60baf56cec198f36f3856e1de1e7580da700e48ce92ca1`, birleşik referans `b5e20f42ea20b904f73ce2c8cd50b3c271f9e5405ad39815c2cbd61da6b7fb92` kimliğindedir.

Yerel raporlar: `compare-tq2_z6ii` (math debug), `compare-889giagp` (math release), `compare-sc8wywfj` (birleşik debug), `compare-wvhin971` (birleşik release) ve `verify-z4jw4eod` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.rotateClockwise90` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. Trigonometrik rotasyonlar, yön dönüşümleri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
