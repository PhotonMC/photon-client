# P01.b — Vec3 rotateClockwise90 doğrulaması

`src/math.rs`, Java `Vec3.rotateClockwise90()` metodunu karşılar. Rust adaptörü sonucu `(-z, y, x)` bileşenleriyle üretir; Java oracle doğrudan `Vec3.rotateClockwise90` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 983 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **983/983** math senaryosunda eşleşti.
- Beş yeni senaryoda eksen permütasyonu, negatif bileşenler, signed zero, subnormal/en büyük sonlu değerler ve Infinity/NaN taşınması karşılaştırıldı.
- Birleşik altı aile corpus'u **1485/1485** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1485 senaryoluk Java referansını yeniden doğruladı.

Math referansı `420532c31a46dc73c7456731fb356a539ca36511c8ae3b916c7b882d3703a8e5`, birleşik referans `7868dda2fe1dd0a7fbb9e78e734b7ecf0ffc505e37763257267a72556cdbc671` kimliğindedir.

Yerel raporlar: `compare-wkwr1b75` (math debug), `compare-4mcmj101` (math release), `compare-3w52gug0` (birleşik debug), `compare-09rh6rbh` (birleşik release) ve `verify-dcqd08ai` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.rotateClockwise90` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. Trigonometrik rotasyonlar, yön dönüşümleri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
