# P01.b — Vec3 distanceToSqr doğrulaması

`src/math.rs`, Java `Vec3.distanceToSqr(Vec3)` ve `Vec3.distanceToSqr(double, double, double)` overload'larını karşılayan iki Rust metodunu içerir. Java oracle doğrudan iki overload'u çağırır; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1335 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1335/1335** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, kaynak toplama sırası, signed zero ve overflow/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1837/1837** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1837 senaryoluk Java referansını yeniden doğruladı.

Math referansı `8956f11f5ab0ba4c81a6aa2c11af8623a498bbb78a550a1b013695afc5029da2`, birleşik referans `f90c38c2c4e4eb83aec92229706d7f6b47bc90a128aa4908ed60897e8279a0d9` kimliğindedir.

Yerel raporlar: `compare-0dy1p_jx` (math debug), `compare-fdzx3gij` (math release), `compare-gridp_gxu` (birleşik debug), `compare-u8zwfn4r` (birleşik release) ve `verify-yhwxvq8l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 mesafe karesi sorgusuyla sınırlıdır. `distanceTo`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
