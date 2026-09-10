# P01.b — Vec3 relative doğrulaması

`src/math.rs`, Java `Vec3.relative(Direction, double)` metodunu karşılar. Rust adaptörü yön normalini uzaklıkla birleştirir; Java oracle doğrudan `Vec3.relative` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2023 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2023/2023** math senaryosunda eşleşti.
- Altı yeni senaryoda EAST/WEST, UP/DOWN ve NORTH/SOUTH yönleri; signed zero, subnormal, sonlu sınır, Infinity, overflow ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2525/2525** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2525 senaryoluk Java referansını yeniden doğruladı.

Math referansı `1a217debf64171ea8f99e220ffd6aca1130ecceef3e6015b82f0c12a4e3f9f0d`, birleşik referans `38386c8d844bb2560f83f62571433316aca08b176233a98979cccf45ec22a8d5` kimliğindedir. Math karşılaştırma raporları `compare-4sx1lm2m` (debug) ve `compare-m3ib6sjn` (release); birleşik raporlar `compare-czvhou3w` (debug) ve `compare-oz19j96e` (release), Java doğrulaması `verify-nukwd4yc` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.relative(Direction, double)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Direction.getUnitVec3i` normalinin diğer tüketicileri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
