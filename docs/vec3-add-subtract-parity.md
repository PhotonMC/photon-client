# P01.b — Vec3 add/subtract overload doğrulaması

`src/math.rs`, Java `Vec3.add` ve `Vec3.subtract` overload'larını karşılayan altı Rust metodu içerir. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 685 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **685/685** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3, scalar ve üç bileşenli overload zincirleri; signed zero, Infinity ve NaN girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1139/1139** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1139 senaryoluk Java referansını yeniden doğruladı.

Math referansı `420fd67254c578e987c7ec5b975c1360f16d2eb8e7d91134b1b7f1f645635be2`, birleşik referans `d2ca721ca4ab536a692660afc465bae364927288154e3b38676eb7e5c86fa0f8` kimliğindedir.

Yerel raporlar: `compare-2oxv9wmk` (math debug), `compare-54arzdz4` (math release), `compare-3tby77d3` (birleşik debug), `compare-nscg_s6g` (birleşik release) ve `verify-bf1nxkx8` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
