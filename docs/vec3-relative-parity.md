# P01.b — Vec3 relative doğrulaması

`src/math.rs`, Java `Vec3.relative(Direction, double)` metodunu karşılar. Rust adaptörü yön normalini uzaklıkla birleştirir; Java oracle doğrudan `Vec3.relative` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 747 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **747/747** math senaryosunda eşleşti.
- Altı yeni senaryoda EAST/WEST, UP/DOWN ve NORTH/SOUTH yönleri; signed zero, subnormal, sonlu sınır, Infinity, overflow ve NaN payload sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1201/1201** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1201 senaryoluk Java referansını yeniden doğruladı.

Math referansı `6a81202124ebb62fb52fe67c65508a75f8eab5935a82637586267fcd663b1d5a`, birleşik referans `faa59d430a7f5ffebd3b634a6e1cb6970d92ff920794ed661c7eaf6175f68fb7` kimliğindedir. Math karşılaştırma raporları `compare-id7epnq0` (debug) ve `compare-otdg_h7s` (release); birleşik raporlar `compare-loq4vhwu` (debug) ve `compare-d1k2vrim` (release), Java doğrulaması `verify-cbpvrvrp` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.relative(Direction, double)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Direction.getUnitVec3i` normalinin diğer tüketicileri ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
