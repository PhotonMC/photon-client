# P01.d — VoxelShape toString doğrulaması

`src/shape.rs`, Java `VoxelShape.toString()` davranışını `VoxelShape::to_string_java` ile uygular. `examples/behavior_adapter.rs`, `to_string` sorgusunu bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 741 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **741/741** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1195/1195** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda empty, grid shape, bilimsel double gösterimi, sonsuzluk ve NaN metinleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1195 senaryoluk Java referansını yeniden doğruladı.

Math referansı `76ec7e6f558a6c9ddbeb7cfe4d528ae99ab6fa83980eeb2c8bda1376494a158f`, birleşik referans `7eb966aeeec47bb875a064f29ee8e61b18ab59b1678b26c179ead90046713129` kimliğindedir.

Yerel raporlar: `compare-bgegqdoc` (math debug), `compare-lkhuntdt` (math release), `compare-crkt4gor` (birleşik debug), `compare-7us5i_vu` (birleşik release) ve `verify-j0qpg9v6` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, Java shape üreticileriyle oluşturulan beş sentetik shape ve bunların `toString()` sonuçlarıyla sınırlıdır. Daha geniş Java sayı biçimi matrisi bu dilime eklenmedi; shape bounds dışındaki metin üreten sınıflar ayrı sözleşmelerdir.
