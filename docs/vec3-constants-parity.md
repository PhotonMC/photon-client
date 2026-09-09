# P01.b — Vec3 sabitleri doğrulaması

`src/math.rs`, Java `Vec3.ZERO`, `X_AXIS`, `Y_AXIS` ve `Z_AXIS` sabitlerini associated constant olarak sunar. Java oracle sabitleri doğrudan okur; Rust adaptörü aynı değerleri üretim API’sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1113 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1113/1113** math senaryosunda eşleşti.
- Dört yeni senaryoda `ZERO` ve üç pozitif birim eksen sabitinin f64 bileşenleri karşılaştırıldı.
- Birleşik altı aile corpus’u **1615/1615** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1615 senaryoluk Java referansını yeniden doğruladı.

Math referansı `23d8549371228c5fe370021e1df1ce61a2d1869ad767cf1a029bcbf1930b5ac9`, birleşik referans `747c3645458af2ae3b9de41407461996409d550b8fca05e990e18b3b1983f461` kimliklerindedir.

Yerel raporlar: `compare-y28xdodh` (math debug), `compare-_7cfq3in` (math release), `compare-cxv1an3r` (birleşik debug), `compare-1jjs0aun` (birleşik release) ve `verify-71u5jh9x` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt dört özgün sabit sorgusu ve Vec3 snapshot sonuçlarıyla sınırlıdır. Sabitlerin oyun davranışındaki tüm tüketicileri sonraki istemci dilimleridir.
