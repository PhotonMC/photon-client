# P01.b — Vec3 distanceToSqr doğrulaması

`src/math.rs`, Java `Vec3.distanceToSqr(Vec3)` ve `Vec3.distanceToSqr(double, double, double)` overload'larını karşılayan iki Rust metodunu içerir. Java oracle doğrudan iki overload'u çağırır; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 977 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **977/977** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, kaynak toplama sırası, signed zero ve overflow/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1479/1479** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1479 senaryoluk Java referansını yeniden doğruladı.

Math referansı `fc20607e30d3e5c68224006103ae9dd1fce364a83ec76ae93deae73d073194b8`, birleşik referans `65cbf0b19b2c96be41cf3bbad7266d1e118047e47e641f45d1473f7a741de148` kimliğindedir.

Yerel raporlar: `compare-89j99zq5` (math debug), `compare-8bts9s4m` (math release), `compare-88a_ixm3` (birleşik debug), `compare-sk_17ilr` (birleşik release) ve `verify-yeb5s_0e` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 mesafe karesi sorgusuyla sınırlıdır. `distanceTo`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
