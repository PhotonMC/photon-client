# P01.b — Vec3 sabitleri doğrulaması

`src/math.rs`, Java `Vec3.ZERO`, `X_AXIS`, `Y_AXIS` ve `Z_AXIS` sabitlerini associated constant olarak sunar. Java oracle sabitleri doğrudan okur; Rust adaptörü aynı değerleri üretim API’sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1987 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1987/1987** math senaryosunda eşleşti.
- Dört yeni senaryoda `ZERO` ve üç pozitif birim eksen sabitinin f64 bileşenleri karşılaştırıldı.
- Birleşik altı aile corpus’u **2489/2489** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2489 senaryoluk Java referansını yeniden doğruladı.

Math referansı `665bdda6827e71f559f4e9a959ed126f3d3e055594f3b677359c51d0d3a74e16`, birleşik referans `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc` kimliklerindedir.

Yerel raporlar: `compare-zt_h_egg` (math debug), `compare-7afcvx21` (math release), `compare-251aed0_` (birleşik debug), `compare-jqj2kta_` (birleşik release) ve `verify-csdh4b83` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt dört özgün sabit sorgusu ve Vec3 snapshot sonuçlarıyla sınırlıdır. Sabitlerin oyun davranışındaki tüm tüketicileri sonraki istemci dilimleridir.
