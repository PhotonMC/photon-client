# P01.d — VoxelShape toString doğrulaması

`src/shape.rs`, Java `VoxelShape.toString()` davranışını `VoxelShape::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` sorgusunu bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 725 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **725/725** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1179/1179** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda empty, grid shape, bilimsel double gösterimi, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1179 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ba2c0a50dc0b1df009707b281c6734a888c59ecbfee1929dc86125f395920582`, birleşik referans `3be9baa80520a67f762a0d1221d9e4a40892597f1dc8ccba80afd18b3fe8689b` kimliğindedir.

Yerel raporlar: `compare-7nv0ftym` (math debug), `compare-omc6avwk` (math release), `compare-cogkg_y3` (birleşik debug), `compare-vj0crhe_` (birleşik release) ve `verify-kco3t37r` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java shape üreticileriyle oluşturulan beş sentetik shape ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi bu dilime eklenmedi; shape bounds dışındaki metin üreten sınıflar ayrı sözleşmelerdir.
