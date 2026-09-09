# P01.d — VoxelShape equals doğrulaması

`src/shape.rs`, Java `VoxelShape.equals(Object)` nesne kimliğini `VoxelShape::equals_java` ile uygular. Singleton üreticileri ve aynı nesneyi döndüren shape yolları kimlik token'ını paylaşır. `examples/behavior_adapter.rs`, `object_equals` ve `object_equals_constants` sorgularını bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 705 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **705/705** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1159/1159** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni senaryoda ayrı shape nesneleri, aynı geometri, `EMPTY`, `BLOCK`, `INFINITY`, threshold ve move kimlikleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1159 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ee70cefd63c52a2329bae4fec46f29bfb59aefb64f7f48829c4a516dc7685fb9`, birleşik referans `eb9f0578dc53caaa0fab43aebfbe945ab957c5f68d0e3eb2d790a5f4fc1c92f2` kimliğindedir.

Yerel raporlar: `compare-g26jdt9a` (math debug), `compare-wlw80e9f` (math release), `compare-l4qzoucn` (birleşik debug), `compare-ixiuyjvh` (birleşik release) ve `verify-o1ktysgq` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, `VoxelShape` operandlarıyla yapılan Java eşitlik çağrılarını kapsar. Java metodunun `null` veya başka bir `Object` türüyle çağrılması Rust tip sınırında ayrı bir giriş olarak modellenmedi; geometrik eşitlik `Shapes.equal` üzerinden ayrı tutuldu.
