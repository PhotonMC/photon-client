# P01.b — Vec3 rotateClockwise90 doğrulaması

`src/math.rs`, Java `Vec3.rotateClockwise90()` metodunu karşılar. Rust adaptörü sonucu `(-z, y, x)` bileşenleriyle üretir; Java oracle doğrudan `Vec3.rotateClockwise90` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 883 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **883/883** math senaryosunda eşleşti.
- Beş yeni senaryoda eksen permütasyonu, negatif bileşenler, signed zero, subnormal/en büyük sonlu değerler ve Infinity/NaN taşınması karşılaştırıldı.
- Birleşik altı aile corpus'u **1337/1337** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1337 senaryoluk Java referansını yeniden doğruladı.

Math referansı `d84c6e390b3e5f2b889fc1898cfe68d63d515aaf6b2301fd91c23a404f1948d3`, birleşik referans `f01f56d2abc01b2e6a0ba297a21b7dd3142493d30df1c618e38bf0a52a708975` kimliğindedir.

Yerel raporlar: `compare-7zvwnrbm` (math debug), `compare-on7ckea` (math release), `compare-4l5i3tbl` (birleşik debug), `compare-f9mdmv2u` (birleşik release) ve `verify-zae5dspu` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.rotateClockwise90` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. Trigonometrik rotasyonlar, yön dönüşümleri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
