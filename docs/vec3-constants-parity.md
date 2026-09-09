# P01.b — Vec3 sabitleri doğrulaması

`src/math.rs`, Java `Vec3.ZERO`, `X_AXIS`, `Y_AXIS` ve `Z_AXIS` sabitlerini associated constant olarak sunar. Java oracle sabitleri doğrudan okur; Rust adaptörü aynı değerleri üretim API’sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 887 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **887/887** math senaryosunda eşleşti.
- Dört yeni senaryoda `ZERO` ve üç pozitif birim eksen sabitinin f64 bileşenleri karşılaştırıldı.
- Birleşik altı aile corpus’u **1341/1341** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1341 senaryoluk Java referansını yeniden doğruladı.

Math referansı `676119520815e68dd9f4f120c8201463ce27aa4921910181223d0cdb38c939ec`, birleşik referans `971b4d305a95fb8e346f40a2edb704ac7f42751d32b8193786c4eb6aae63d1fb` kimliklerindedir.

Yerel raporlar: `compare-d1bitq7h` (math debug), `compare-ldlepu8u` (math release), `compare-kuzkqdbl` (birleşik debug), `compare-n3x2kj9p` (birleşik release) ve `verify-jbdhw9mf` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt dört özgün sabit sorgusu ve Vec3 snapshot sonuçlarıyla sınırlıdır. Sabitlerin oyun davranışındaki tüm tüketicileri sonraki istemci dilimleridir.
