# P01.b — Vec3 distanceToSqr doğrulaması

`src/math.rs`, Java `Vec3.distanceToSqr(Vec3)` ve `Vec3.distanceToSqr(double, double, double)` overload'larını karşılayan iki Rust metodunu içerir. Java oracle doğrudan iki overload'u çağırır; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1193 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1193/1193** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, kaynak toplama sırası, signed zero ve overflow/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1695/1695** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1695 senaryoluk Java referansını yeniden doğruladı.

Math referansı `3a6a2f5a3d8ff6b65e565413ca95aeeb802f3ca965620ed874e898bcc52c5d61`, birleşik referans `5f58cd401556c40d3fc3eb7c3e511a8dbc2553c3390034c888e648d9d8a3ad4c` kimliğindedir.

Yerel raporlar: `compare-dgq7mp_7` (math debug), `compare-2hmy86uj` (math release), `compare-06i9lguy` (birleşik debug), `compare-r6guli2r` (birleşik release) ve `verify-k_vt31c0` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 mesafe karesi sorgusuyla sınırlıdır. `distanceTo`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
