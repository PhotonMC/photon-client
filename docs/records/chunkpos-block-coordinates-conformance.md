# P01.c — ChunkPos blok koordinatı yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, bir ChunkPos'un kapsadığı blok x/z aralığının minimum, orta ve maksimum uçlarını ve verilen offset ile hesaplanan blok koordinatlarını taşımaktır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:104-126` içindeki `getMiddleBlockX/Z`, `getMinBlockX/Z`, `getMaxBlockX/Z` ve `getBlockX/Z` metotları ile `core/SectionPos.java:128-134` içindeki section-to-block dönüşümü incelendi. Oracle, Java metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde `ChunkPos`'un region, BlockPos üretimi, contains, distance, range ve diğer yardımcıları ölçülmez. Blok koordinatı girdileri signed decimal i32 metinlerinden oluşturulur.

## Davranış sözleşmesi

- `getMinBlockX/Z`, chunk koordinatını 4 bit sola kaydırarak kapsanan blok aralığının alt ucunu üretir.
- `getMiddleBlockX/Z`, aynı dönüşümde offset 8'i kullanır.
- `getMaxBlockX/Z`, aynı dönüşümde offset 15'i kullanır.
- `getBlockX/Z(offset)`, section-to-block dönüşümüne verilen offset'i ekler.
- Java int sol kaydırma ve toplama taşmaları i32 içinde sarar; negatif chunk koordinatları taban bölme yerine bit kaydırma semantiğini korur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| Blok koordinatı yardımcıları | 6 | Köken, pozitif/negatif chunk, aralık sınırı, shift taşması ve offset taşması |

Girdiler `family: "math"`, `operation: "chunkpos_block_coordinates"`, `x`, `z` ve `offset` alanlarını kullanır. Tüm sonuçlar `value` altında minimum, orta, maksimum ve offset'li x/z koordinatlarını signed decimal string olarak taşır.

## Rust karşılığı

`crates/photon-core/src/binary.rs` içindeki `ChunkPos::get_block_x`, `get_block_z`, `get_min_block_x`, `get_min_block_z`, `get_middle_block_x`, `get_middle_block_z`, `get_max_block_x` ve `get_max_block_z` Java'nın 4 bit sola kaydırma ve i32 wrapping add yolunu korur. `examples/behavior_adapter.rs` bu API'yi JSONL senaryosuna bağlar; `tests/behavior/java/MathOracle.java` Java çıktısını doğrudan gözlemler.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs`, Java `ChunkPos` blok koordinatı yardımcılarını aynı i32 kaydırma ve toplama kurallarıyla karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik x/z/offset girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- ChunkPos blok koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Minimum, orta, maksimum ve offset'li koordinatlar x/z sırasını korudu; negatif chunk koordinatları, shift taşması ve offset toplama taşması Java i32 sarma davranışıyla eşleşti.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, `ChunkPos`'un blok x/z alt-orta-üst uçlarını ve offset'li blok koordinatlarını kapsar. Region koordinatları, `BlockPos` üretimi, contains/distance/range yardımcıları, stream/JSON codec'leri ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
