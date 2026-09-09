# P01.d — AABB toString doğrulaması

`src/aabb.rs`, Java `AABB.toString()` davranışını `Aabb::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` adımını bu API'ye taşır; `VoxelShape.toString()` da aynı AABB biçimlendirme yolunu kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 983 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **983/983** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1485/1485** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda normal endpoint, bilimsel double gösterimi, signed zero, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1485 senaryoluk Java referansını yeniden doğruladı.

Math referansı `420532c31a46dc73c7456731fb356a539ca36511c8ae3b916c7b882d3703a8e5`, birleşik referans `7868dda2fe1dd0a7fbb9e78e734b7ecf0ffc505e37763257267a72556cdbc671` kimliğindedir.

Yerel raporlar: `compare-wkwr1b75` (math debug), `compare-4mcmj101` (math release), `compare-3w52gug0` (birleşik debug), `compare-09rh6rbh` (birleşik release) ve `verify-dcqd08ai` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java constructor'ı ile oluşturulan beş sentetik AABB ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi ve diğer sınıfların metin biçimleri ayrı sözleşmelerdir.
