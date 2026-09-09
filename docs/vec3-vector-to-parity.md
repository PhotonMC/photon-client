# P01.b — Vec3 vectorTo doğrulaması

`src/math.rs`, Java `Vec3.vectorTo(Vec3)` metodunu karşılayan `Vec3::vector_to` metodunu içerir. Java oracle doğrudan `vectorTo` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 665 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **665/665** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu fark, ardışık hedef değişimi, signed zero, double taşması, Infinity ve NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1119/1119** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1119 senaryoluk Java referansını yeniden doğruladı.

Math referansı `b4f21f456b1ed4b8f6d8d04534a21132ed8c550c6cc3e8f152c696a895bf9de0`, birleşik referans `3b2bdc05bfdf0766c7665b84746689cdc5d05a2b2c751cf2d9a7ceaf66690145` kimliğindedir.

Yerel raporlar: `compare-2pd_izg6` (math debug), `compare-k81tuucm` (math release), `compare-ybb_vjnz` (birleşik debug), `compare-xxko56va` (birleşik release) ve `verify-p5bur1pm` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distance` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
