# P01.b — Vec3 koordinat yardımcıları doğrulaması

`src/math.rs`, Java `Vec3` sınıfının BlockPos tabanlı koordinat yardımcılarını karşılar. Java oracle, `atLowerCornerOf`, `atLowerCornerWithOffset`, `atCenterOf`, `atBottomCenterOf` ve `upFromBottomCenterOf` çağrılarını doğrudan çalıştırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 771 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **771/771** math senaryosunda eşleşti.
- Altı yeni senaryoda alt köşe, üç bileşenli offset, merkez, taban merkezi, y offset'i ve signed zero/subnormal/NaN offset sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1225/1225** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1225 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ffefa3418c35a5dbf05a16b8c6bb664f38665423ada7b039029bea24f0e3f8ac`, birleşik referans `c8d3f97682c7983d7226696163a3943fe4ef6723170037ad5205d10ed0494654` kimliğindedir. Math karşılaştırma raporları `compare-nyuxtnwe` (debug) ve `compare-c83i9uhh` (release); birleşik raporlar `compare-2v85255z` (debug) ve `compare-4_se5ekn` (release), Java doğrulaması `verify-q8mzlbkq` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik BlockPos girdisinde beş Java üreticisinin Vec3 x, y, z, length ve length squared f64 ham bit snapshot'larıyla sınırlıdır. `Vec3` constructor overload'ları ve daha geniş BlockPos/SectionPos davranışları ayrı dilimlerdir.
