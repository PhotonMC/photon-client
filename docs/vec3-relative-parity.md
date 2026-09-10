# P01.b — Vec3 relative doğrulaması

`src/math.rs`, Java `Vec3.relative(Direction, double)` metodunu karşılar. Rust adaptörü yön normalini uzaklıkla birleştirir; Java oracle doğrudan `Vec3.relative` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1398 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1398/1398** math senaryosunda eşleşti.
- Altı yeni senaryoda EAST/WEST, UP/DOWN ve NORTH/SOUTH yönleri; signed zero, subnormal, sonlu sınır, Infinity, overflow ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1900/1900** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1900 senaryoluk Java referansını yeniden doğruladı.

Math referansı `4cc46837e07b2a435f9a79ffddebd09d7fed0ad74e1310e64453de08cb4bf4bd`, birleşik referans `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea` kimliğindedir. Math karşılaştırma raporları `compare-xljwmsb7` (debug) ve `compare-s_oqvkq3` (release); birleşik raporlar `compare-uf5y7y3a` (debug) ve `compare-wrj9a6ex` (release), Java doğrulaması `verify-5ju3ok97` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.relative(Direction, double)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Direction.getUnitVec3i` normalinin diğer tüketicileri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
