# P01.b — Vec3 koordinat yardımcıları doğrulaması

`src/math.rs`, Java `Vec3` sınıfının BlockPos tabanlı koordinat yardımcılarını karşılar. Java oracle, `atLowerCornerOf`, `atLowerCornerWithOffset`, `atCenterOf`, `atBottomCenterOf` ve `upFromBottomCenterOf` çağrılarını doğrudan çalıştırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1052 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1052/1052** math senaryosunda eşleşti.
- Altı yeni senaryoda alt köşe, üç bileşenli offset, merkez, taban merkezi, y offset'i ve signed zero/subnormal/NaN offset sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1554/1554** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1554 senaryoluk Java referansını yeniden doğruladı.

Math referansı `d4f17c1024ea8a511a4611971e222dcdaa1a00ec6c1baa442420a0f051c914cc`, birleşik referans `f615eef08df36fc7705e9f9da886cfc498964867c630a79cb4bf7f15f3c0bae1` kimliğindedir. Math karşılaştırma raporları `compare-xj4m01k2` (debug) ve `compare-ix4n2052` (release); birleşik raporlar `compare-e0lm9mqh` (debug) ve `compare-pp8moc03` (release), Java doğrulaması `verify-r4qtgn3b` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik BlockPos girdisinde beş Java üreticisinin Vec3 x, y, z, length ve length squared f64 ham bit snapshot'larıyla sınırlıdır. `Vec3` constructor overload'ları ve daha geniş BlockPos/SectionPos davranışları ayrı dilimlerdir.
