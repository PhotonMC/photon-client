# P01.b — Vec3 koordinat yardımcıları doğrulaması

`src/math.rs`, Java `Vec3` sınıfının BlockPos tabanlı koordinat yardımcılarını karşılar. Java oracle, `atLowerCornerOf`, `atLowerCornerWithOffset`, `atCenterOf`, `atBottomCenterOf` ve `upFromBottomCenterOf` çağrılarını doğrudan çalıştırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 831 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **831/831** math senaryosunda eşleşti.
- Altı yeni senaryoda alt köşe, üç bileşenli offset, merkez, taban merkezi, y offset'i ve signed zero/subnormal/NaN offset sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1285/1285** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1285 senaryoluk Java referansını yeniden doğruladı.

Math referansı `1fc10b2ffb1b5d99516b0450dd2f29125f7d5307a496734b48d04cceeb6766b0`, birleşik referans `0c3a5294e06d5c7794e3841f9940b5f92d3361d48ecf89a9c24fcc9ec1cda700` kimliğindedir. Math karşılaştırma raporları `compare-qkr5g9de` (debug) ve `compare-csmtu7u6` (release); birleşik raporlar `compare-vpfl308t` (debug) ve `compare-6zh1lj9w` (release), Java doğrulaması `verify-6d8una0v` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik BlockPos girdisinde beş Java üreticisinin Vec3 x, y, z, length ve length squared f64 ham bit snapshot'larıyla sınırlıdır. `Vec3` constructor overload'ları ve daha geniş BlockPos/SectionPos davranışları ayrı dilimlerdir.
