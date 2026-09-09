# P01.d — VoxelShape toString doğrulaması

`src/shape.rs`, Java `VoxelShape.toString()` davranışını `VoxelShape::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` sorgusunu bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 670 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **670/670** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1124/1124** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda empty, grid shape, bilimsel double gösterimi, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1124 senaryoluk Java referansını yeniden doğruladı.

Math referansı `233afc7fe403783bb0e269123225910816ff5ec1075c4bf10cdf8bbbcdc2853f`, birleşik referans `7463d45cd07a5207a5d29e6be51ea489c541efadb966932e595b98fdda8d1940` kimliğindedir.

Yerel raporlar: `compare-ra03eisz` (math debug), `compare-up24hdi8` (math release), `compare-rgbylm8r` (birleşik debug), `compare-445klgjk` (birleşik release) ve `verify-bt555rfc` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java shape üreticileriyle oluşturulan beş sentetik shape ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi bu dilime eklenmedi; shape bounds dışındaki metin üreten sınıflar ayrı sözleşmelerdir.
