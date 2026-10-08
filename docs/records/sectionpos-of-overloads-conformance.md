# P01.c — SectionPos of(...) overloadları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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

`crates/photon-core/src/binary.rs` içindeki `SectionPos::from_block_pos`, `from_chunk_pos` ve `from_position`, Java overload'larının mevcut int/double section dönüşüm yollarını taşır. Aynı türdeki `SectionPos::new` `of(int, int, int)` için, `SectionPos::unpack` ise `of(long)` için packed karşılıktır. `examples/behavior_adapter.rs` dört modu JSONL'e bağlar; `tests/behavior/java/MathOracle.java` Java overload'larını doğrudan çağırır.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs`, SectionPos'u BlockPos, ChunkPos ve double Position girdilerinden üretme yollarını ve packed SectionPos açılımını Java ile aynı section koordinatlarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos `of(...)` veri overload'ları için 6 özgün senaryo hazırlandı.
- BlockPos signed i32 sınırları, ChunkPos + sectionY aktarımı, packed alan sign extension'ı, negatif/pozitif double floor ve özel double değerleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.of(BlockPos)`, `of(ChunkPos, int)`, `of(Position)` ve `of(long)` yollarının son SectionPos snapshot'ıyla sınırlıdır. `of(EntityAccess)`, `bottomOf(ChunkAccess)`, gerçek entity/chunk üretim zincirleri, stream codec ve `of(int, int, int)` için ayrı çağrı gözlemi bu dilimin dışındadır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
