# P01.c — ChunkPos BlockPos yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, bir `ChunkPos` içindeki offset'lerden `BlockPos` üretmek, orta blok konumunu ve dünya başlangıç konumunu almak ve verilen bir `BlockPos`'un chunk'ın yatay sınırları içinde olup olmadığını kontrol etmektir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:152-190` içindeki `getBlockAt`, `getMiddleBlockPosition`, `contains` ve `getWorldPosition` metotları incelendi. Oracle, Java metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde `BlockPos` packing'i, chunk region/stream/JSON codec'leri, block koordinatı yardımcılarının kendisi, range/distance ve istemciye bağlı chunk davranışları ölçülmez. `contains`, Java'daki gibi yalnızca x/z eksenlerini karşılaştırır; query y değeri sonucu etkilemez.

## Davranış sözleşmesi

- `getBlockAt(x, y, z)`, chunk'ın block x/z dönüşümüne verilen offset'leri uygular ve y değerini doğrudan taşır.
- `getMiddleBlockPosition(y)`, block x/z için 8 offset'ini ve verilen y değerini kullanır.
- `getWorldPosition()`, chunk'ın minimum block x/z koordinatlarını y=0 ile birlikte döndürür.
- `contains(BlockPos)`, minimum ve maksimum block x/z uçlarını kapsayıcı karşılaştırır; y eksenini yok sayar.
- Block koordinatı dönüşümü ve offset toplaması Java i32 sarma davranışını korur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| ChunkPos → BlockPos yardımcıları | 6 | Köken, pozitif/negatif chunk, sınır, offset taşması ve shift taşması |

Girdiler `family: "math"`, `operation: "chunkpos_block_positions"`, chunk `x`/`z`, block offset'leri, y ve `contains` query listesini kullanır. `value` altında `block_at`, `middle_block_position`, `world_position` ve sıralı containment sonuçları taşınır.

## Rust karşılığı

`crates/voxel-core/src/binary.rs` içindeki `ChunkPos::get_block_at`, `get_middle_block_position`, `get_world_position` ve `contains` Java'nın BlockPos üretim sırasını, inclusive yatay sınırlarını ve i32 wrapping aritmetiğini korur. `examples/behavior_adapter.rs` bu API'yi JSONL senaryosuna bağlar; `tests/behavior/java/MathOracle.java` Java çıktısını doğrudan gözlemler.


## Client-parity doğrulaması

`crates/voxel-core/src/binary.rs`, Java `ChunkPos` BlockPos yardımcılarını aynı block offset, middle/world position ve yatay containment kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik chunk, offset, y ve query girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos → BlockPos yardımcıları için 6 özgün senaryo hazırlandı.
- Offset'li BlockPos üretimi, middle/world position sonuçları, inclusive x/z sınırları, Y'den bağımsız containment ve taşma yolları Java i32 davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'tan BlockPos üretimini, middle/world konumlarını ve yatay containment sorgusunu kapsar. BlockPos packing'i, ChunkPos region/codec yardımcıları, range/distance, dünya/chunk yaşam döngüsü ve istemciye bağlı davranışlar bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
