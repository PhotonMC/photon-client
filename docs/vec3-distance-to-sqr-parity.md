# P01.b — Vec3 distanceToSqr doğrulaması

`src/math.rs`, Java `Vec3.distanceToSqr(Vec3)` ve `Vec3.distanceToSqr(double, double, double)` overload'larını karşılayan iki Rust metodunu içerir. Java oracle doğrudan iki overload'u çağırır; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 735 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **735/735** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, kaynak toplama sırası, signed zero ve overflow/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1189/1189** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1189 senaryoluk Java referansını yeniden doğruladı.

Math referansı `94a6025ce62400506c0736719b21030eeb6b7d69b6ad0d5d1f2e9c8c3af90491`, birleşik referans `e1537e5e73915e7c752cf9b583382949d05feaca1a439d67f330b3513cb7483e` kimliğindedir.

Yerel raporlar: `compare-iadkiiub` (math debug), `compare-f0hxm0vh` (math release), `compare-wlecne0c` (birleşik debug), `compare-uvna2fit` (birleşik release) ve `verify-ihxsw2pd` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 mesafe karesi sorgusuyla sınırlıdır. `distanceTo`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
