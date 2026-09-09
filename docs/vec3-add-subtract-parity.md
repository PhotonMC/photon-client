# P01.b — Vec3 add/subtract overload doğrulaması

`src/math.rs`, Java `Vec3.add` ve `Vec3.subtract` overload'larını karşılayan altı Rust metodu içerir. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1001 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1001/1001** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3, scalar ve üç bileşenli overload zincirleri; signed zero, Infinity ve NaN girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1503/1503** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1503 senaryoluk Java referansını yeniden doğruladı.

Math referansı `7d65abdf4967bf0f7585e849fc4e6783027eaef1ef313c8b7d22e85922955d4a`, birleşik referans `2411fed740f5d2548545a86cee882fb42daaf0d9fbededf1b224dce54a261a5d` kimliğindedir.

Yerel raporlar: `compare-12czaqmk` (math debug), `compare-fm0qnyjh` (math release), `compare-sakmrbet` (birleşik debug), `compare-pd6md7hs` (birleşik release) ve `verify-5t2xrfy6` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
