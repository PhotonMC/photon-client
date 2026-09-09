# P01.b — Vec3 rotateClockwise90 doğrulaması

`src/math.rs`, Java `Vec3.rotateClockwise90()` metodunu karşılar. Rust adaptörü sonucu `(-z, y, x)` bileşenleriyle üretir; Java oracle doğrudan `Vec3.rotateClockwise90` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 959 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **959/959** math senaryosunda eşleşti.
- Beş yeni senaryoda eksen permütasyonu, negatif bileşenler, signed zero, subnormal/en büyük sonlu değerler ve Infinity/NaN taşınması karşılaştırıldı.
- Birleşik altı aile corpus'u **1461/1461** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1461 senaryoluk Java referansını yeniden doğruladı.

Math referansı `35103d673698aa0dcef8706322bcfccb0a54e6545c75f67308f6f27f498c922d`, birleşik referans `2fb61f9fd2ce23842225b3592a92ff5a162ad0567561f4606996f2ddbc58842d` kimliğindedir.

Yerel raporlar: `compare-cuylkpl7` (math debug), `compare-iuha2872` (math release), `compare-o2800g_u` (birleşik debug), `compare-7ryigcxt` (birleşik release) ve `verify-jnzrnmg2` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.rotateClockwise90` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. Trigonometrik rotasyonlar, yön dönüşümleri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
