# P01.d — VoxelShape equals sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `VoxelShape.equals(Object)` metodunun nesne kimliği davranışı ile `Shapes` singleton üreticilerinin bu kimliğe etkisini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:313-316` içindeki override'ın temel nesne eşitliğine yönlendirdiği, `Shapes.java:23-50` ve `Shapes.java:92-95` içindeki `BLOCK`, `INFINITY` ve `EMPTY` üretim yollarıyla birlikte incelendi. `MathOracle`, shape'leri Java constructor/üreticileriyle oluşturup doğrudan eşitlik sorgularını çalıştırır.

## Davranış sözleşmesi

- Aynı `VoxelShape` nesnesi kendisine eşittir.
- Ayrı nesneler, hücreleri ve sınırları aynı olsa bile eşit değildir. Geometrik karşılaştırma için ayrı `Shapes.equal` metodu kullanılır.
- `Shapes.empty()` ve `Shapes.block()` paylaşılan singleton nesneleri döndürür; eşik nedeniyle boş dönen shape ve tam birim blok da bu kimlikleri kullanır.
- `Shapes.INFINITY` tek paylaşılan shape'tir. Aynı koordinatlarla sonradan oluşturulan bir sonsuz kutu bu singleton ile eşit değildir.
- Java'da boş shape'i taşımak `EMPTY` singleton'ını döndürür; dolu shape'i sıfır delta ile taşımak yeni nesne üretir.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `cube` | Aynı hücre geometrisini üreten iki ayrı bounded shape | Kendisi, aynı geometri ve farklı geometri karşılaştırmaları |
| `array` | Birim küp dışındaki koordinatlarla oluşturulan iki shape | Kendisi, aynı geometri ve farklı geometri karşılaştırmaları |
| `constants` | `EMPTY`, `BLOCK`, `INFINITY`, eşik ve move yolları | Singleton kimliği sonuçları |

Girdiler `family: "math"`, `operation: "shape"` ve `kind: "object_equals"` veya `kind: "object_equals_constants"` alanlarını kullanır. `kind: "equal"` mevcut geometrik `Shapes.equal` sözleşmesidir ve bu davranıştan ayrıdır.

## Rust karşılığı

`VoxelShape::equals_java`, shape nesnesi kimliğini paylaşılan bir `Arc` token'ı ile karşılaştırır. `Shapes::empty`, `Shapes::block` ve `Shapes::infinity` bu token'ları static `OnceLock` değerleri üzerinden paylaşır; Java'da aynı nesneyi döndüren iç yollar `Clone` ile token'ı korur.


## Client-parity doğrulaması

`crates/voxel-core/src/shape.rs`, Java `VoxelShape.equals(Object)` nesne kimliğini `VoxelShape::equals_java` ile uygular. Singleton üreticileri ve aynı nesneyi döndüren shape yolları kimlik token'ını paylaşır. `examples/behavior_adapter.rs`, `object_equals` ve `object_equals_constants` sorgularını bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni senaryoda ayrı shape nesneleri, aynı geometri, `EMPTY`, `BLOCK`, `INFINITY`, threshold ve move kimlikleri karşılaştırıldı.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-o1ktysgq` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, `VoxelShape` operandlarıyla yapılan Java eşitlik çağrılarını kapsar. Java metodunun `null` veya başka bir `Object` türüyle çağrılması Rust tip sınırında ayrı bir giriş olarak modellenmedi; geometrik eşitlik `Shapes.equal` üzerinden ayrı tutuldu.
