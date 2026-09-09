# P01.b — Vec3 relative doğrulaması

`src/math.rs`, Java `Vec3.relative(Direction, double)` metodunu karşılar. Rust adaptörü yön normalini uzaklıkla birleştirir; Java oracle doğrudan `Vec3.relative` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 959 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **959/959** math senaryosunda eşleşti.
- Altı yeni senaryoda EAST/WEST, UP/DOWN ve NORTH/SOUTH yönleri; signed zero, subnormal, sonlu sınır, Infinity, overflow ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1461/1461** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1461 senaryoluk Java referansını yeniden doğruladı.

Math referansı `35103d673698aa0dcef8706322bcfccb0a54e6545c75f67308f6f27f498c922d`, birleşik referans `2fb61f9fd2ce23842225b3592a92ff5a162ad0567561f4606996f2ddbc58842d` kimliğindedir. Math karşılaştırma raporları `compare-cuylkpl7` (debug) ve `compare-iuha2872` (release); birleşik raporlar `compare-o2800g_u` (debug) ve `compare-7ryigcxt` (release), Java doğrulaması `verify-jnzrnmg2` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.relative(Direction, double)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Direction.getUnitVec3i` normalinin diğer tüketicileri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
