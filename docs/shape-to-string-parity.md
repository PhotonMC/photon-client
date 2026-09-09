# P01.d — VoxelShape toString doğrulaması

`src/shape.rs`, Java `VoxelShape.toString()` davranışını `VoxelShape::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` sorgusunu bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 883 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **883/883** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1337/1337** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda empty, grid shape, bilimsel double gösterimi, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1337 senaryoluk Java referansını yeniden doğruladı.

Math referansı `d84c6e390b3e5f2b889fc1898cfe68d63d515aaf6b2301fd91c23a404f1948d3`, birleşik referans `f01f56d2abc01b2e6a0ba297a21b7dd3142493d30df1c618e38bf0a52a708975` kimliğindedir.

Yerel raporlar: `compare-7zvwnrbm` (math debug), `compare-on7ckea` (math release), `compare-4l5i3tbl` (birleşik debug), `compare-f9mdmv2u` (birleşik release) ve `verify-zae5dspu` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java shape üreticileriyle oluşturulan beş sentetik shape ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi bu dilime eklenmedi; shape bounds dışındaki metin üreten sınıflar ayrı sözleşmelerdir.
