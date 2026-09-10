# P01.b — Vec3 sabitleri doğrulaması

`src/math.rs`, Java `Vec3.ZERO`, `X_AXIS`, `Y_AXIS` ve `Z_AXIS` sabitlerini associated constant olarak sunar. Java oracle sabitleri doğrudan okur; Rust adaptörü aynı değerleri üretim API’sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1506 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1506/1506** math senaryosunda eşleşti.
- Dört yeni senaryoda `ZERO` ve üç pozitif birim eksen sabitinin f64 bileşenleri karşılaştırıldı.
- Birleşik altı aile corpus’u **2008/2008** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2008 senaryoluk Java referansını yeniden doğruladı.

Math referansı `521c517003287d78e7dee051e04062b28c1fdcf7965213cb691cb7e1e0a3f9df`, birleşik referans `8a534e59c23e5da8152cd10fc0f559f6e05cac44a30f5600b40c9a6b5a45ce57` kimliklerindedir.

Yerel raporlar: `compare-wn40y3iw` (math debug), `compare-9phkvs2k` (math release), `compare-a415_00_` (birleşik debug), `compare-djb85w4w` (birleşik release) ve `verify-nqxvpkjm` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt dört özgün sabit sorgusu ve Vec3 snapshot sonuçlarıyla sınırlıdır. Sabitlerin oyun davranışındaki tüm tüketicileri sonraki istemci dilimleridir.
