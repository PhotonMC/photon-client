# P01.b — Vec3 koordinat yardımcıları doğrulaması

`src/math.rs`, Java `Vec3` sınıfının BlockPos tabanlı koordinat yardımcılarını karşılar. Java oracle, `atLowerCornerOf`, `atLowerCornerWithOffset`, `atCenterOf`, `atBottomCenterOf` ve `upFromBottomCenterOf` çağrılarını doğrudan çalıştırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1843 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1843/1843** math senaryosunda eşleşti.
- Altı yeni senaryoda alt köşe, üç bileşenli offset, merkez, taban merkezi, y offset'i ve signed zero/subnormal/NaN offset sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2345/2345** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2345 senaryoluk Java referansını yeniden doğruladı.

Math referansı `dfeb55b1b61cbc7649df7ae8af88469cbf7480a194892aacbcf8e5e5b93806bd`, birleşik referans `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576` kimliğindedir. Math karşılaştırma raporları `compare-5_avciij` (debug) ve `compare-e6e6agr4` (release); birleşik raporlar `compare-gvo3eind` (debug) ve `compare-wmabbl5t` (release), Java doğrulaması `verify-vzenwlxd` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik BlockPos girdisinde beş Java üreticisinin Vec3 x, y, z, length ve length squared f64 ham bit snapshot'larıyla sınırlıdır. `Vec3` constructor overload'ları ve daha geniş BlockPos/SectionPos davranışları ayrı dilimlerdir.
