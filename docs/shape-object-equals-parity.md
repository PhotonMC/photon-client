# P01.d — VoxelShape equals doğrulaması

`src/shape.rs`, Java `VoxelShape.equals(Object)` nesne kimliğini `VoxelShape::equals_java` ile uygular. Singleton üreticileri ve aynı nesneyi döndüren shape yolları kimlik token'ını paylaşır. `examples/behavior_adapter.rs`, `object_equals` ve `object_equals_constants` sorgularını bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 685 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **685/685** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1139/1139** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni senaryoda ayrı shape nesneleri, aynı geometri, `EMPTY`, `BLOCK`, `INFINITY`, threshold ve move kimlikleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 1139 senaryoluk Java referansını yeniden doğruladı.

Math referansı `420fd67254c578e987c7ec5b975c1360f16d2eb8e7d91134b1b7f1f645635be2`, birleşik referans `d2ca721ca4ab536a692660afc465bae364927288154e3b38676eb7e5c86fa0f8` kimliğindedir.

Yerel raporlar: `compare-2oxv9wmk` (math debug), `compare-54arzdz4` (math release), `compare-3tby77d3` (birleşik debug), `compare-nscg_s6g` (birleşik release) ve `verify-o1ktysgq` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, `VoxelShape` operandlarıyla yapılan Java eşitlik çağrılarını kapsar. Java metodunun `null` veya başka bir `Object` türüyle çağrılması Rust tip sınırında ayrı bir giriş olarak modellenmedi; geometrik eşitlik `Shapes.equal` üzerinden ayrı tutuldu.
