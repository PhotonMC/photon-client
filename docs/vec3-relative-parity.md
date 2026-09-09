# P01.b — Vec3 relative doğrulaması

`src/math.rs`, Java `Vec3.relative(Direction, double)` metodunu karşılar. Rust adaptörü yön normalini uzaklıkla birleştirir; Java oracle doğrudan `Vec3.relative` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 831 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **831/831** math senaryosunda eşleşti.
- Altı yeni senaryoda EAST/WEST, UP/DOWN ve NORTH/SOUTH yönleri; signed zero, subnormal, sonlu sınır, Infinity, overflow ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1285/1285** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1285 senaryoluk Java referansını yeniden doğruladı.

Math referansı `1fc10b2ffb1b5d99516b0450dd2f29125f7d5307a496734b48d04cceeb6766b0`, birleşik referans `0c3a5294e06d5c7794e3841f9940b5f92d3361d48ecf89a9c24fcc9ec1cda700` kimliğindedir. Math karşılaştırma raporları `compare-qkr5g9de` (debug) ve `compare-csmtu7u6` (release); birleşik raporlar `compare-vpfl308t` (debug) ve `compare-6zh1lj9w` (release), Java doğrulaması `verify-6d8una0v` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.relative(Direction, double)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Direction.getUnitVec3i` normalinin diğer tüketicileri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
