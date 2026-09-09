# P01.d — VoxelShape equals doğrulaması

`src/shape.rs`, Java `VoxelShape.equals(Object)` nesne kimliğini `VoxelShape::equals_java` ile uygular. Singleton üreticileri ve aynı nesneyi döndüren shape yolları kimlik token'ını paylaşır. `examples/behavior_adapter.rs`, `object_equals` ve `object_equals_constants` sorgularını bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 869 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **869/869** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1323/1323** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni senaryoda ayrı shape nesneleri, aynı geometri, `EMPTY`, `BLOCK`, `INFINITY`, threshold ve move kimlikleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1323 senaryoluk Java referansını yeniden doğruladı.

Math referansı `6d26ec313384e0e1de60baf56cec198f36f3856e1de1e7580da700e48ce92ca1`, birleşik referans `b5e20f42ea20b904f73ce2c8cd50b3c271f9e5405ad39815c2cbd61da6b7fb92` kimliğindedir.

Yerel raporlar: `compare-tq2_z6ii` (math debug), `compare-889giagp` (math release), `compare-sc8wywfj` (birleşik debug), `compare-wvhin971` (birleşik release) ve `verify-o1ktysgq` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, `VoxelShape` operandlarıyla yapılan Java eşitlik çağrılarını kapsar. Java metodunun `null` veya başka bir `Object` türüyle çağrılması Rust tip sınırında ayrı bir giriş olarak modellenmedi; geometrik eşitlik `Shapes.equal` üzerinden ayrı tutuldu.
