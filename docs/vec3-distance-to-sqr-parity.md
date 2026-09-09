# P01.b — Vec3 distanceToSqr doğrulaması

`src/math.rs`, Java `Vec3.distanceToSqr(Vec3)` ve `Vec3.distanceToSqr(double, double, double)` overload'larını karşılayan iki Rust metodunu içerir. Java oracle doğrudan iki overload'u çağırır; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 947 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **947/947** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, kaynak toplama sırası, signed zero ve overflow/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1449/1449** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1449 senaryoluk Java referansını yeniden doğruladı.

Math referansı `173698891dc93629abbcd5ca3642ca5af04a5ddb10ffcbaa0610cf03306c320e`, birleşik referans `304e1c88da43eaa3ceb24663a0ab71ddb6fb1f5487c293a9f94fb7224b94c363` kimliğindedir.

Yerel raporlar: `compare-auuk33g2` (math debug), `compare-w_2b2455` (math release), `compare-tjsmsqxw` (birleşik debug), `compare-sks6l8sa` (birleşik release) ve `verify-egp3gvx_` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 mesafe karesi sorgusuyla sınırlıdır. `distanceTo`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
