# P01.b — Vec3 vectorTo doğrulaması

`src/math.rs`, Java `Vec3.vectorTo(Vec3)` metodunu karşılayan `Vec3::vector_to` metodunu içerir. Java oracle doğrudan `vectorTo` çağrısını yapar; Rust adaptörü aynı f64 bit girdisini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 935 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **935/935** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu fark, ardışık hedef değişimi, signed zero, double taşması, Infinity ve NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1437/1437** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1437 senaryoluk Java referansını yeniden doğruladı.

Math referansı `b5430468b5bb806bec96f53c626329209858940d1a04da9bbfbedb16ab6b2dc0`, birleşik referans `2a0e476c9429ade902407539ef06621159d2eed0b5010504338eecbf1e77e14a` kimliğindedir.

Yerel raporlar: `compare-96_s862r` (math debug), `compare-cbwrzmb5` (math release), `compare-fn36ocz_` (birleşik debug), `compare-a_hggp11` (birleşik release) ve `verify-k4za48of` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
