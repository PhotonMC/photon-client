# P01.d — VoxelShape equals doğrulaması

`src/shape.rs`, Java `VoxelShape.equals(Object)` nesne kimliğini `VoxelShape::equals_java` ile uygular. Singleton üreticileri ve aynı nesneyi döndüren shape yolları kimlik token'ını paylaşır. `examples/behavior_adapter.rs`, `object_equals` ve `object_equals_constants` sorgularını bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1506 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1506/1506** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2008/2008** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni senaryoda ayrı shape nesneleri, aynı geometri, `EMPTY`, `BLOCK`, `INFINITY`, threshold ve move kimlikleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 2008 senaryoluk Java referansını yeniden doğruladı.

Math referansı `521c517003287d78e7dee051e04062b28c1fdcf7965213cb691cb7e1e0a3f9df`, birleşik referans `8a534e59c23e5da8152cd10fc0f559f6e05cac44a30f5600b40c9a6b5a45ce57` kimliğindedir.

Yerel raporlar: `compare-wn40y3iw` (math debug), `compare-9phkvs2k` (math release), `compare-a415_00_` (birleşik debug), `compare-djb85w4w` (birleşik release) ve `verify-o1ktysgq` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, `VoxelShape` operandlarıyla yapılan Java eşitlik çağrılarını kapsar. Java metodunun `null` veya başka bir `Object` türüyle çağrılması Rust tip sınırında ayrı bir giriş olarak modellenmedi; geometrik eşitlik `Shapes.equal` üzerinden ayrı tutuldu.
