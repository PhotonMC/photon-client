# P01.c — SectionPos of(...) overloadları sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, SectionPos'un mevcut saf veri türlerinden üretilen `of(BlockPos)`, `of(ChunkPos, int)`, `of(Position)` ve `of(long)` yollarıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:41-67`, `core/Position.java:3-9`, `world/phys/Vec3.java:18,65-69` ve `world/level/ChunkPos.java:20-20` incelendi. Oracle dört Java overload yolunu doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

`of(BlockPos)`, BlockPos'un x/y/z bileşenlerini int `blockToSectionCoord` yoluna verir. `of(ChunkPos, int)`, ChunkPos x/z değerlerini ve verilen sectionY değerini doğrudan SectionPos bileşenlerine taşır. `of(Position)`, Position'ın double x/y/z değerlerini double `blockToSectionCoord` yoluyla floor edip section koordinatlarına çevirir; Vec3 bu arayüzün testteki somut değeridir. `of(long)`, packed SectionPos düğümünün signed 22/20/22 bit x/y/z alanlarını açar.

Position yolunda negatif kesirler aşağı yuvarlanır. Java double→int daraltması NaN için sıfır, sonsuz ve aralık dışı değerler için signed int sınırları üretir; section dönüşümü ardından dört bit arithmetic shift uygular. Packed yolunda alan sign extension'ı korunur.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| BlockPos overload | 2 | Pozitif/negatif section sınırları ve signed i32 uçları |
| ChunkPos overload | 1 | Negatif x, pozitif z ve verilen negatif sectionY |
| Packed long overload | 1 | Tüm bitleri bir olan düğümde alan sign extension'ı |
| Position overload | 2 | Negatif/pozitif kesirler ile en büyük sonlu, NaN ve negatif sonsuz double |

Girdiler `family: "math"`, `operation: "sectionpos_of_overloads"` ve `mode` alanlarını kullanır. `blockpos` ve `chunkpos` yollarında koordinatlar signed decimal i32 dizgeleridir; `chunkpos` ayrıca `section_y` taşır. `position` yolunda x/y/z 16 haneli unsigned hexadecimal f64 bit dizgeleridir. `packed` yolunda `section_node` 16 haneli unsigned hexadecimal packed değerdir. `result.value`, x/y/z signed decimal koordinatlarını ve `packed` unsigned lowercase hexadecimal SectionPos düğümünü taşır.

Bu dilim `of(EntityAccess)` ve `bottomOf(ChunkAccess)` overload'larını içermez; mevcut saf istemci veri modelinde bu bağımlılıkların karşılığı yoktur. `of(int, int, int)` yolu önceki SectionPos senaryolarında constructor/koordinat üretimiyle zaten kullanılır.

## Rust karşılığı

`src/binary.rs` içindeki `SectionPos::from_block_pos`, `from_chunk_pos` ve `from_position`, Java overload'larının mevcut int/double section dönüşüm yollarını taşır. Aynı türdeki `SectionPos::new` `of(int, int, int)` için, `SectionPos::unpack` ise `of(long)` için packed karşılıktır. `examples/behavior_adapter.rs` dört modu JSONL'e bağlar; `tests/behavior/java/MathOracle.java` Java overload'larını doğrudan çağırır.
