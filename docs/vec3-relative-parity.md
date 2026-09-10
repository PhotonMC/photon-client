# P01.b — Vec3 relative doğrulaması

`src/math.rs`, Java `Vec3.relative(Direction, double)` metodunu karşılar. Rust adaptörü yön normalini uzaklıkla birleştirir; Java oracle doğrudan `Vec3.relative` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1422 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1422/1422** math senaryosunda eşleşti.
- Altı yeni senaryoda EAST/WEST, UP/DOWN ve NORTH/SOUTH yönleri; signed zero, subnormal, sonlu sınır, Infinity, overflow ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1924/1924** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1924 senaryoluk Java referansını yeniden doğruladı.

Math referansı `cc96d24cc80c682020b8b8d28795f7b434dad5677010e1608fc1b6e2a730e5d7`, birleşik referans `a346583dc8bd7a5d672fe737c86d9febc8a2861cc99d0c486416f41813870b7c` kimliğindedir. Math karşılaştırma raporları `compare-ox0b8e23` (debug) ve `compare-8yyrdrg5` (release); birleşik raporlar `compare-xbim8rgs` (debug) ve `compare-kg978z18` (release), Java doğrulaması `verify-tw8nqf_h` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.relative(Direction, double)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Direction.getUnitVec3i` normalinin diğer tüketicileri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
