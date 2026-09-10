# P01.d — VoxelShape toString doğrulaması

`src/shape.rs`, Java `VoxelShape.toString()` davranışını `VoxelShape::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` sorgusunu bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1548 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1548/1548** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2050/2050** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda empty, grid shape, bilimsel double gösterimi, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 2050 senaryoluk Java referansını yeniden doğruladı.

Math referansı `8d3d3f4690e190086d874fa8e58bdb837efcc4649b07170de736639c38e7aa7d`, birleşik referans `0d4c76950a37451c226bb147399cfa08349c81c96e4a8ea2b85e47696f0caf11` kimliğindedir.

Yerel raporlar: `compare-m8bramo3` (math debug), `compare-20kwpz7m` (math release), `compare-hmu92wq0` (birleşik debug), `compare-ha45dyno` (birleşik release) ve `verify-46sjfnqt` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java shape üreticileriyle oluşturulan beş sentetik shape ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi bu dilime eklenmedi; shape bounds dışındaki metin üreten sınıflar ayrı sözleşmelerdir.
