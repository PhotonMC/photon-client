# P01.d — VoxelShape equals doğrulaması

`src/shape.rs`, Java `VoxelShape.equals(Object)` nesne kimliğini `VoxelShape::equals_java` ile uygular. Singleton üreticileri ve aynı nesneyi döndüren shape yolları kimlik token'ını paylaşır. `examples/behavior_adapter.rs`, `object_equals` ve `object_equals_constants` sorgularını bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1843 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1843/1843** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2345/2345** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni senaryoda ayrı shape nesneleri, aynı geometri, `EMPTY`, `BLOCK`, `INFINITY`, threshold ve move kimlikleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 2345 senaryoluk Java referansını yeniden doğruladı.

Math referansı `dfeb55b1b61cbc7649df7ae8af88469cbf7480a194892aacbcf8e5e5b93806bd`, birleşik referans `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576` kimliğindedir.

Yerel raporlar: `compare-5_avciij` (math debug), `compare-e6e6agr4` (math release), `compare-gvo3eind` (birleşik debug), `compare-wmabbl5t` (birleşik release) ve `verify-o1ktysgq` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, `VoxelShape` operandlarıyla yapılan Java eşitlik çağrılarını kapsar. Java metodunun `null` veya başka bir `Object` türüyle çağrılması Rust tip sınırında ayrı bir giriş olarak modellenmedi; geometrik eşitlik `Shapes.equal` üzerinden ayrı tutuldu.
