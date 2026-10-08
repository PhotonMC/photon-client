# P01.c — SectionPos blocksInside sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **3 Java davranış testi hazırlanmıştır**. Kapsam, bir SectionPos'ın içerdiği 16×16×16 BlockPos değerlerinin inclusive traversal sırasıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:160-182,234-235` ve `.data/minecraft/source/net/minecraft/core/BlockPos.java:392-417` incelendi. Oracle `SectionPos.of(int, int, int)` ile section oluşturur, `blocksInside()` stream'ini tüketir ve her BlockPos'ı gözlemler; beklenen sonuçlar Rust çıktısından türetilmez.

`blocksInside`, section'ın minimum ve maksimum blok uçlarını `BlockPos.betweenClosedStream` yoluna verir. Her eksen 16 değer içerir. `BlockPos` cursor'ı x eksenini hızlı değiştirir; x satırı tamamlanınca y, y dilimi tamamlanınca z ilerler. Stream tüketilirken mutable cursor değerleri her öğe için JSON snapshot'ına kopyalanır.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| `blocksInside` | 3 | Köken, negatif section koordinatları ve section→block sınırları |

Girdiler `family: "math"`, `operation: "sectionpos_blocks_inside"` ve signed decimal i32 `x`, `y`, `z` alanlarını kullanır. `result.value.positions`, 4096 BlockPos değerinin x/y/z signed decimal snapshot'larını Java traversal sırasıyla içerir.

Bu sözleşme sonlu 16×16×16 section çıktısına odaklanır. Java stream tembelliği, spliterator özellikleri ve mutable BlockPos nesnesinin kimliği bu dilime dahil değildir.

## Rust karşılığı

`crates/photon-core/src/binary.rs` içindeki `SectionPos::blocks_inside`, `min_block_*` ve `max_block_*` uçlarını Java i32 wrapping aritmetiğiyle hesaplayıp aynı x-fastest/y-next/z-next sırada 4096 `BlockPos` üretir. `examples/behavior_adapter.rs` sonucu JSONL operation'ına bağlar; `tests/behavior/java/MathOracle.java` Java stream'ini karşılaştırma oracle'ı olarak tüketir.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs`, SectionPos `blocksInside()` akışını Java ile aynı 16×16×16 inclusive BlockPos küpüne ve x/y/z traversal sırasına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos blocksInside için 3 özgün senaryo hazırlandı.
- Her senaryo 4096 BlockPos çıktısında origin, negatif section ve section→block sınırlarını doğrudan Java/Rust API'leriyle karşılaştırdı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.blocksInside()` ve kullandığı `BlockPos.betweenClosedStream` yolunun sabit 4096 öğelik çıktısıyla sınırlıdır. Genel BlockPos range overload'ları, stream laziness/spliterator özellikleri ve `aroundAndAtBlockPos` akışı ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
