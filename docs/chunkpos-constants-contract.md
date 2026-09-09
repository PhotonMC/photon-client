# P01.c — ChunkPos public sabitleri sözleşmesi

Bu dilimde **4 Java davranış testi hazırlanmıştır**. Kapsam, `ChunkPos` sınıfının public static sabitlerinin doğrudan değerleridir.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/world/level/ChunkPos.java:32-43` incelendi. Oracle sabitleri doğrudan okur; beklenen sonuçlar Rust çıktısından türetilmez.

- `ZERO`, x ve z koordinatları sıfır olan ChunkPos'tur.
- `INVALID_CHUNK_POS`, iki `1_875_066` koordinatının ChunkPos packed long biçimindeki sentinel değeridir ve unsigned lowercase hex olarak taşınır.
- `REGION_SIZE`, bir region içindeki chunk eksen uzunluğu olarak `32` değerini taşır.
- `REGION_MAX_INDEX`, region içindeki son local index olarak `31` değerini taşır.

Private katsayılar, sabitlerin başka Java metodlarındaki tüketicileri ve invalid sentinel'ın oyun akışındaki anlamı bu dilimin dışındadır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `zero` | `name: "zero"` | `result.value.chunk_pos` içindeki x/z decimal string alanları |
| `invalid_chunk_pos` | `name: "invalid_chunk_pos"` | `result.value.packed` içindeki unsigned lowercase hex packed long |
| `region_size` | `name: "region_size"` | `result.value.value` içindeki decimal string |
| `region_max_index` | `name: "region_max_index"` | `result.value.value` içindeki decimal string |

Girdiler `family: "math"` ve `operation: "chunkpos_constant"` alanlarını kullanır.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::ZERO`, `INVALID_CHUNK_POS`, `REGION_SIZE` ve `REGION_MAX_INDEX` associated constant'ları Java değerlerini taşır. `examples/behavior_adapter.rs` sabiti doğrudan Rust API'sinden sorgular; `tests/behavior/java/MathOracle.java` aynı adı Java sabitine bağlar.
