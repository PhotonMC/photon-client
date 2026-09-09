# P01.d — AABB toString doğrulaması

`src/aabb.rs`, Java `AABB.toString()` davranışını `Aabb::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` adımını bu API'ye taşır; `VoxelShape.toString()` da aynı AABB biçimlendirme yolunu kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 887 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **887/887** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1341/1341** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint, bilimsel double gösterimi, signed zero, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1341 senaryoluk Java referansını yeniden doğruladı.

Math referansı `676119520815e68dd9f4f120c8201463ce27aa4921910181223d0cdb38c939ec`, birleşik referans `971b4d305a95fb8e346f40a2edb704ac7f42751d32b8193786c4eb6aae63d1fb` kimliğindedir.

Yerel raporlar: `compare-d1bitq7h` (math debug), `compare-ldlepu8u` (math release), `compare-kuzkqdbl` (birleşik debug), `compare-n3x2kj9p` (birleşik release) ve `verify-jbdhw9mf` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java constructor'ı ile oluşturulan beş sentetik AABB ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi ve diğer sınıfların metin biçimleri ayrı sözleşmelerdir.
