# P01.b — Vec3 rotateClockwise90 doğrulaması

`src/math.rs`, Java `Vec3.rotateClockwise90()` metodunu karşılar. Rust adaptörü sonucu `(-z, y, x)` bileşenleriyle üretir; Java oracle doğrudan `Vec3.rotateClockwise90` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1623 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1623/1623** math senaryosunda eşleşti.
- Beş yeni senaryoda eksen permütasyonu, negatif bileşenler, signed zero, subnormal/en büyük sonlu değerler ve Infinity/NaN taşınması karşılaştırıldı.
- Birleşik altı aile corpus'u **2125/2125** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2125 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ad2753dc2b6d8419f59cf4874d459be1bbd0128324ea6d37e1afd0b11e7854cf`, birleşik referans `14062308d34164b4d537528e94d89e360ce28e02bc1488fc00ccd7a15390b0eb` kimliğindedir.

Yerel raporlar: `compare-ny3j5bmv` (math debug), `compare-8wh6otww` (math release), `compare-us5kn_kw` (birleşik debug), `compare-qnl_96jt` (birleşik release) ve `verify-m1vz27n6` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.rotateClockwise90` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. Trigonometrik rotasyonlar, yön dönüşümleri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
