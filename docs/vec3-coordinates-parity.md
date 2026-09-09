# P01.b — Vec3 koordinat yardımcıları doğrulaması

`src/math.rs`, Java `Vec3` sınıfının BlockPos tabanlı koordinat yardımcılarını karşılar. Java oracle, `atLowerCornerOf`, `atLowerCornerWithOffset`, `atCenterOf`, `atBottomCenterOf` ve `upFromBottomCenterOf` çağrılarını doğrudan çalıştırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1046 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1046/1046** math senaryosunda eşleşti.
- Altı yeni senaryoda alt köşe, üç bileşenli offset, merkez, taban merkezi, y offset'i ve signed zero/subnormal/NaN offset sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1548/1548** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1548 senaryoluk Java referansını yeniden doğruladı.

Math referansı `bd99eadfa488c44d624d410a441fe8b99d4485c7236dd276609af1edc3394f3b`, birleşik referans `7a79765a9b449a8ea9c09f049a1ab917958fff879c31b1058ce2f4b8dd3208ab` kimliğindedir. Math karşılaştırma raporları `compare-m8v98wom` (debug) ve `compare-4i19c_i3` (release); birleşik raporlar `compare-p7m9abjy` (debug) ve `compare-vvlbkmvf` (release), Java doğrulaması `verify-_w4izisu` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik BlockPos girdisinde beş Java üreticisinin Vec3 x, y, z, length ve length squared f64 ham bit snapshot'larıyla sınırlıdır. `Vec3` constructor overload'ları ve daha geniş BlockPos/SectionPos davranışları ayrı dilimlerdir.
