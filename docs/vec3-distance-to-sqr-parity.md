# P01.b — Vec3 distanceToSqr doğrulaması

`src/math.rs`, Java `Vec3.distanceToSqr(Vec3)` ve `Vec3.distanceToSqr(double, double, double)` overload'larını karşılayan iki Rust metodunu içerir. Java oracle doğrudan iki overload'u çağırır; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 753 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **753/753** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, kaynak toplama sırası, signed zero ve overflow/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1207/1207** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1207 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9f7c6c7c4510b51d789726e1fbdacf38416646884cd1bcd8ed50b971bc7d6c32`, birleşik referans `e2ca4584024ee04709e92930122695a6bdc10c32c7e1a4c6b90f89808b9bfd54` kimliğindedir.

Yerel raporlar: `compare-88hucivw` (math debug), `compare-puyjkq8y` (math release), `compare-o4r8ph3e` (birleşik debug), `compare-umiwejve` (birleşik release) ve `verify-acev0jw9` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 mesafe karesi sorgusuyla sınırlıdır. `distanceTo`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
