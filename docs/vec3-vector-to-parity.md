# P01.b — Vec3 vectorTo doğrulaması

`src/math.rs`, Java `Vec3.vectorTo(Vec3)` metodunu karşılayan `Vec3::vector_to` metodunu içerir. Java oracle doğrudan `vectorTo` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 771 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **771/771** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu fark, ardışık hedef değişimi, signed zero, double taşması, Infinity ve NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1225/1225** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1225 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ffefa3418c35a5dbf05a16b8c6bb664f38665423ada7b039029bea24f0e3f8ac`, birleşik referans `c8d3f97682c7983d7226696163a3943fe4ef6723170037ad5205d10ed0494654` kimliğindedir.

Yerel raporlar: `compare-nyuxtnwe` (math debug), `compare-c83i9uhh` (math release), `compare-2v85255z` (birleşik debug), `compare-4_se5ekn` (birleşik release) ve `verify-q8mzlbkq` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
