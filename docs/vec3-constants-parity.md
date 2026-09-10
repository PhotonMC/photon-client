# P01.b — Vec3 sabitleri doğrulaması

`src/math.rs`, Java `Vec3.ZERO`, `X_AXIS`, `Y_AXIS` ve `Z_AXIS` sabitlerini associated constant olarak sunar. Java oracle sabitleri doğrudan okur; Rust adaptörü aynı değerleri üretim API’sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1548 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1548/1548** math senaryosunda eşleşti.
- Dört yeni senaryoda `ZERO` ve üç pozitif birim eksen sabitinin f64 bileşenleri karşılaştırıldı.
- Birleşik altı aile corpus’u **2050/2050** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2050 senaryoluk Java referansını yeniden doğruladı.

Math referansı `8d3d3f4690e190086d874fa8e58bdb837efcc4649b07170de736639c38e7aa7d`, birleşik referans `0d4c76950a37451c226bb147399cfa08349c81c96e4a8ea2b85e47696f0caf11` kimliklerindedir.

Yerel raporlar: `compare-m8bramo3` (math debug), `compare-20kwpz7m` (math release), `compare-hmu92wq0` (birleşik debug), `compare-ha45dyno` (birleşik release) ve `verify-46sjfnqt` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt dört özgün sabit sorgusu ve Vec3 snapshot sonuçlarıyla sınırlıdır. Sabitlerin oyun davranışındaki tüm tüketicileri sonraki istemci dilimleridir.
