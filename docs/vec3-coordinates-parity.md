# P01.b — Vec3 koordinat yardımcıları doğrulaması

`src/math.rs`, Java `Vec3` sınıfının BlockPos tabanlı koordinat yardımcılarını karşılar. Java oracle, `atLowerCornerOf`, `atLowerCornerWithOffset`, `atCenterOf`, `atBottomCenterOf` ve `upFromBottomCenterOf` çağrılarını doğrudan çalıştırır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1398 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1398/1398** math senaryosunda eşleşti.
- Altı yeni senaryoda alt köşe, üç bileşenli offset, merkez, taban merkezi, y offset'i ve signed zero/subnormal/NaN offset sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1900/1900** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1900 senaryoluk Java referansını yeniden doğruladı.

Math referansı `4cc46837e07b2a435f9a79ffddebd09d7fed0ad74e1310e64453de08cb4bf4bd`, birleşik referans `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea` kimliğindedir. Math karşılaştırma raporları `compare-xljwmsb7` (debug) ve `compare-s_oqvkq3` (release); birleşik raporlar `compare-uf5y7y3a` (debug) ve `compare-wrj9a6ex` (release), Java doğrulaması `verify-5ju3ok97` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik BlockPos girdisinde beş Java üreticisinin Vec3 x, y, z, length ve length squared f64 ham bit snapshot'larıyla sınırlıdır. `Vec3` constructor overload'ları ve daha geniş BlockPos/SectionPos davranışları ayrı dilimlerdir.
