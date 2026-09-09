# P01.d — VoxelShape toString doğrulaması

`src/shape.rs`, Java `VoxelShape.toString()` davranışını `VoxelShape::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` sorgusunu bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1113 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1113/1113** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1615/1615** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda empty, grid shape, bilimsel double gösterimi, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1615 senaryoluk Java referansını yeniden doğruladı.

Math referansı `23d8549371228c5fe370021e1df1ce61a2d1869ad767cf1a029bcbf1930b5ac9`, birleşik referans `747c3645458af2ae3b9de41407461996409d550b8fca05e990e18b3b1983f461` kimliğindedir.

Yerel raporlar: `compare-y28xdodh` (math debug), `compare-_7cfq3in` (math release), `compare-cxv1an3r` (birleşik debug), `compare-1jjs0aun` (birleşik release) ve `verify-71u5jh9x` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java shape üreticileriyle oluşturulan beş sentetik shape ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi bu dilime eklenmedi; shape bounds dışındaki metin üreten sınıflar ayrı sözleşmelerdir.
