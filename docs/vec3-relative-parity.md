# P01.b — Vec3 relative doğrulaması

`src/math.rs`, Java `Vec3.relative(Direction, double)` metodunu karşılar. Rust adaptörü yön normalini uzaklıkla birleştirir; Java oracle doğrudan `Vec3.relative` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1783 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1783/1783** math senaryosunda eşleşti.
- Altı yeni senaryoda EAST/WEST, UP/DOWN ve NORTH/SOUTH yönleri; signed zero, subnormal, sonlu sınır, Infinity, overflow ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2285/2285** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2285 senaryoluk Java referansını yeniden doğruladı.

Math referansı `c9c0c64d34c86e78a480fb2f63bd3bd5633a857332ad5997d97039c193be5338`, birleşik referans `6251a239b5efd4862d79c5b740dcb208ec9e1932f8a04d0d86c05a044353fd90` kimliğindedir. Math karşılaştırma raporları `compare-1g27oorn` (debug) ve `compare-bkcvw7oi` (release); birleşik raporlar `compare-qmmqnzwk` (debug) ve `compare-ogma6668` (release), Java doğrulaması `verify-1hv399w4` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.relative(Direction, double)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Direction.getUnitVec3i` normalinin diğer tüketicileri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
