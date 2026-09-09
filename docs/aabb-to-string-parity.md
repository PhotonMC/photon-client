# P01.d — AABB toString doğrulaması

`src/aabb.rs`, Java `AABB.toString()` davranışını `Aabb::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` adımını bu API'ye taşır; `VoxelShape.toString()` da aynı AABB biçimlendirme yolunu kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 973 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **973/973** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1475/1475** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint, bilimsel double gösterimi, signed zero, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1475 senaryoluk Java referansını yeniden doğruladı.

Math referansı `5b9ac04dfbc4d0f113189cc9a302a301459740911a59e19bb68a172015f61780`, birleşik referans `6be1cb26871cc94c222742a0c5ccb50c6ef5df50a6b4f33206aeefdd805d2a60` kimliğindedir.

Yerel raporlar: `compare-yzj8g47` (math debug), `compare-0gyul_vz` (math release), `compare-1qxbdgol` (birleşik debug), `compare-9okhsl5l` (birleşik release) ve `verify-v5w4qfr` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java constructor'ı ile oluşturulan beş sentetik AABB ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi ve diğer sınıfların metin biçimleri ayrı sözleşmelerdir.
