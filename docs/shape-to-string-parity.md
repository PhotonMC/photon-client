# P01.d — VoxelShape toString doğrulaması

`src/shape.rs`, Java `VoxelShape.toString()` davranışını `VoxelShape::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` sorgusunu bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 700 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **700/700** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1154/1154** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda empty, grid shape, bilimsel double gösterimi, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1154 senaryoluk Java referansını yeniden doğruladı.

Math referansı `c1800d2e244360700681ce9dd25627eeb76c5b378b61d6c6a9894994bc858ad8`, birleşik referans `040454adf30b9e5c2e8c273080741126bc70be6ae1b1c6d5176978231e9d036a` kimliğindedir.

Yerel raporlar: `compare-yy9k89p5` (math debug), `compare-og6yrygd` (math release), `compare-downo9zd` (birleşik debug), `compare-66zm55sg` (birleşik release) ve `verify-tf9y7j96` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java shape üreticileriyle oluşturulan beş sentetik shape ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi bu dilime eklenmedi; shape bounds dışındaki metin üreten sınıflar ayrı sözleşmelerdir.
