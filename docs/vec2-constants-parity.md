# P01.b — Vec2 sabitleri doğrulaması

`src/math.rs`, Java `Vec2.ZERO`, `ONE`, `UNIT_X`, `NEG_UNIT_X`, `UNIT_Y`, `NEG_UNIT_Y`, `MAX` ve `MIN` sabitlerini associated constant olarak sunar. Java oracle sabitleri doğrudan okur; Rust adaptörü aynı değerleri üretim API’sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1319 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1319/1319** math senaryosunda eşleşti.
- Sekiz yeni senaryoda bütün public Vec2 sabitlerinin f32 bileşenleri, `MAX` sınırı ve `MIN` pozitif subnormal değeri karşılaştırıldı.
- Birleşik altı aile corpus’u **1821/1821** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1821 senaryoluk Java referansını yeniden doğruladı.

Math referansı `85b9ce09336ec312c84a062e772cdb41b6a2e079b6fcb1995200470edf61dd6c`, birleşik referans `01a770901948bb0f5650c8b4e80bf54b513d904c9746232c38b93c03a7546957` kimliklerindedir.

Yerel raporlar: `compare-stkj42d2` (math debug), `compare-e9upg098` (math release), `compare-olmqgxpc` (birleşik debug), `compare-jxg0__tj` (birleşik release) ve `verify-m0natom3` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt sekiz özgün sabit sorgusu ve Vec2 snapshot sonuçlarıyla sınırlıdır. Sabitlerin oyun davranışındaki tüm tüketicileri sonraki istemci dilimleridir.
