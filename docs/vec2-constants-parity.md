# P01.b — Vec2 sabitleri doğrulaması

`src/math.rs`, Java `Vec2.ZERO`, `ONE`, `UNIT_X`, `NEG_UNIT_X`, `UNIT_Y`, `NEG_UNIT_Y`, `MAX` ve `MIN` sabitlerini associated constant olarak sunar. Java oracle sabitleri doğrudan okur; Rust adaptörü aynı değerleri üretim API’sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1783 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1783/1783** math senaryosunda eşleşti.
- Sekiz yeni senaryoda bütün public Vec2 sabitlerinin f32 bileşenleri, `MAX` sınırı ve `MIN` pozitif subnormal değeri karşılaştırıldı.
- Birleşik altı aile corpus’u **2285/2285** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2285 senaryoluk Java referansını yeniden doğruladı.

Math referansı `c9c0c64d34c86e78a480fb2f63bd3bd5633a857332ad5997d97039c193be5338`, birleşik referans `6251a239b5efd4862d79c5b740dcb208ec9e1932f8a04d0d86c05a044353fd90` kimliklerindedir.

Yerel raporlar: `compare-1g27oorn` (math debug), `compare-bkcvw7oi` (math release), `compare-qmmqnzwk` (birleşik debug), `compare-ogma6668` (birleşik release) ve `verify-1hv399w4` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt sekiz özgün sabit sorgusu ve Vec2 snapshot sonuçlarıyla sınırlıdır. Sabitlerin oyun davranışındaki tüm tüketicileri sonraki istemci dilimleridir.
