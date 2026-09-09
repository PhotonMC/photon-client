# P01.d — VoxelShape equals doğrulaması

`src/shape.rs`, Java `VoxelShape.equals(Object)` nesne kimliğini `VoxelShape::equals_java` ile uygular. Singleton üreticileri ve aynı nesneyi döndüren shape yolları kimlik token'ını paylaşır. `examples/behavior_adapter.rs`, `object_equals` ve `object_equals_constants` sorgularını bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1031 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1031/1031** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1533/1533** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni senaryoda ayrı shape nesneleri, aynı geometri, `EMPTY`, `BLOCK`, `INFINITY`, threshold ve move kimlikleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1533 senaryoluk Java referansını yeniden doğruladı.

Math referansı `cbae17cd88ce5b1617d01034d27ad104db9b471eefc8bebf83ed4e8d1dc0aa70`, birleşik referans `338731e44c45c8acc6fbd70e7fe1439cdaa28ec5996f6a4d9517412cc50106a0` kimliğindedir.

Yerel raporlar: `compare-2ftkouxd` (math debug), `compare-g204xxpd` (math release), `compare-ag9x0lbh` (birleşik debug), `compare-sl2ed6ln` (birleşik release) ve `verify-o1ktysgq` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, `VoxelShape` operandlarıyla yapılan Java eşitlik çağrılarını kapsar. Java metodunun `null` veya başka bir `Object` türüyle çağrılması Rust tip sınırında ayrı bir giriş olarak modellenmedi; geometrik eşitlik `Shapes.equal` üzerinden ayrı tutuldu.
