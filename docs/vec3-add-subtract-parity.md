# P01.b — Vec3 add/subtract overload doğrulaması

`src/math.rs`, Java `Vec3.add` ve `Vec3.subtract` overload'larını karşılayan altı Rust metodu içerir. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 983 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **983/983** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3, scalar ve üç bileşenli overload zincirleri; signed zero, Infinity ve NaN girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1485/1485** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1485 senaryoluk Java referansını yeniden doğruladı.

Math referansı `420532c31a46dc73c7456731fb356a539ca36511c8ae3b916c7b882d3703a8e5`, birleşik referans `7868dda2fe1dd0a7fbb9e78e734b7ecf0ffc505e37763257267a72556cdbc671` kimliğindedir.

Yerel raporlar: `compare-wkwr1b75` (math debug), `compare-4mcmj101` (math release), `compare-3w52gug0` (birleşik debug), `compare-09rh6rbh` (birleşik release) ve `verify-dcqd08ai` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
