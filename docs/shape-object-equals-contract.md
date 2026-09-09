# P01.d — VoxelShape equals sözleşmesi

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-object-equals-parity.md)). Kapsam, `VoxelShape.equals(Object)` metodunun nesne kimliği davranışı ile `Shapes` singleton üreticilerinin bu kimliğe etkisini içerir.

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
