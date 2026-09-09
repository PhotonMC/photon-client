# P01.b — Vec3 relative doğrulaması

`src/math.rs`, Java `Vec3.relative(Direction, double)` metodunu karşılar. Rust adaptörü yön normalini uzaklıkla birleştirir; Java oracle doğrudan `Vec3.relative` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 777 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **777/777** math senaryosunda eşleşti.
- Altı yeni senaryoda EAST/WEST, UP/DOWN ve NORTH/SOUTH yönleri; signed zero, subnormal, sonlu sınır, Infinity, overflow ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1231/1231** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1231 senaryoluk Java referansını yeniden doğruladı.

Math referansı `d6723874f1f8073346a0f729f2c06b2e00e2dcd34baad1a33733698d293350d3`, birleşik referans `0f128e8e7fb68abc882891752c85767e02d3b02a6b9f83fff7f87b45ab6f2afe` kimliğindedir. Math karşılaştırma raporları `compare-da6k05zu` (debug) ve `compare-jem8yqw3` (release); birleşik raporlar `compare-r10lr0oe` (debug) ve `compare-rb0k9jx5` (release), Java doğrulaması `verify-04cazxan` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.relative(Direction, double)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Direction.getUnitVec3i` normalinin diğer tüketicileri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
