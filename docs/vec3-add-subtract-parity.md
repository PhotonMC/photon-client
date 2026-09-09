# P01.b — Vec3 add/subtract overload doğrulaması

`src/math.rs`, Java `Vec3.add` ve `Vec3.subtract` overload'larını karşılayan altı Rust metodu içerir. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 660 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **660/660** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3, scalar ve üç bileşenli overload zincirleri; signed zero, Infinity ve NaN girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1114/1114** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1114 senaryoluk Java referansını yeniden doğruladı.

Math referansı `bc87ac5ece4a47cf40d4e562d69ac72206437e2c117cd058a79561ba4b4d149b`, birleşik referans `15dc52b0af0addc9b446d43666846404d2d8e35a7cc7751489632ec703b826b8` kimliğindedir.

Yerel raporlar: `compare-3kg8m9ta` (math debug), `compare-6crwfl2x` (math release), `compare-_35t8r3l` (birleşik debug), `compare-g5oda9d_` (birleşik release) ve `verify-6xepcluv` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distance` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
