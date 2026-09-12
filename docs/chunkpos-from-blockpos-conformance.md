# P01.c — ChunkPos BlockPos'tan containing/pack sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, bir `BlockPos` için onu içeren `ChunkPos` değerini bulmak ve aynı x/z section koordinatlarını Java'nın packed long biçimine çevirmektir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/level/ChunkPos.java:45-47` içindeki `containing(BlockPos)`, `world/level/ChunkPos.java:81-83` içindeki `pack(BlockPos)` ve `core/SectionPos.java:81-83` içindeki int `blockToSectionCoord` yolu incelendi. Oracle, Java metotlarını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Bu dilimde `BlockPos.pack`, `SectionPos` node packing'i, ChunkPos stream/JSON codec'leri ve chunk yaşam döngüsü ölçülmez. `BlockPos` y değeri her iki ChunkPos yolunda da kullanılmaz.

## Davranış sözleşmesi

- `ChunkPos.containing(BlockPos)`, BlockPos x ve z bileşenlerini 4 bit arithmetic right shift ile section/chunk koordinatlarına çevirir.
- Negatif block koordinatları aşağı doğru section'a düşer; dönüşüm Java signed i32 bit davranışını korur.
- `ChunkPos.pack(BlockPos)`, containing sonucunun x koordinatını packed long'un düşük 32 bitine, z koordinatını yüksek 32 bitine koyar.
- Packed long, oracle çıktısında `Long.toUnsignedString(..., 16)` ile küçük harfli, baştaki sıfırları kaldırılmış unsigned hex metin olarak taşınır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler ve çıktı |
| --- | ---: | --- |
| BlockPos → ChunkPos | 6 | Köken, pozitif/negatif section sınırları, mixed koordinatlar ve signed i32 uçları |

Girdiler `family: "math"`, `operation: "chunkpos_from_blockpos"` ve signed decimal i32 `x`, `y`, `z` alanlarını kullanır. `value.containing` altında ChunkPos x/z snapshot'ı, `value.packed` altında Java unsigned hex packed sonucu bulunur.

## Rust karşılığı

`src/binary.rs` içindeki `ChunkPos::containing` ve `ChunkPos::pack_block_pos`, mevcut arithmetic section dönüşümü ve ChunkPos packing yollarını birleştirir. `examples/behavior_adapter.rs` JSONL bağını sağlar; `tests/behavior/java/MathOracle.java` Java `ChunkPos.containing` ve `ChunkPos.pack(BlockPos)` çağrılarını doğrudan gözlemler.


## Client-parity doğrulaması

`src/binary.rs`, Java `ChunkPos.containing(BlockPos)` ve `ChunkPos.pack(BlockPos)` davranışlarını x/z arithmetic shift'i, y'nin dışarıda bırakılması ve packed 32 bit alan yerleşimiyle karşılar. Java oracle metotları doğrudan çağırır; Rust adaptörü aynı sentetik signed BlockPos girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- BlockPos'tan containing/pack için 6 özgün senaryo hazırlandı.
- Pozitif ve negatif 16'lık sınırlar, mixed x/z, signed i32 uçları, y'nin etkisizliği ve packed unsigned hex metni Java sonucu ile eşleşti.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` kayıtlı referans doğrulamasında, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, BlockPos'tan ChunkPos containing ve packed long üretimini kapsar. BlockPos packing'i, SectionPos node packing'i, diğer ChunkPos codec/yardımcıları, range/distance ve istemciye bağlı chunk davranışları bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
