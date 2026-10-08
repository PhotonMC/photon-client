# P01.c — SectionPos aroundAndAtBlockPos sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, `aroundAndAtBlockPos` üç overload'unun verilen block konumu çevresindeki section node callback sırasıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:267-293` incelendi. Oracle coordinates, BlockPos ve packed BlockPos overload'larını doğrudan çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

Metot, block koordinatlarının `-1` ve `+1` komşularını section koordinatına çevirir. Üç aralık da aynı section içindeyse tek node çağrılır. Aksi halde sectionX dış, sectionY orta ve sectionZ iç döngüdür; z en hızlı değişir. Block koordinatı `i32` çıkarma/toplama Java wrapping semantiğiyle yürür.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| coordinates | 3 | Interior, origin corner, x section boundary |
| BlockPos overload | 1 | Three-axis boundary |
| packed overload | 1 | All-one packed BlockPos |
| i32 overflow | 1 | `Integer.MAX_VALUE` x endpoint |

Girdiler `family: "math"`, `operation: "sectionpos_around_and_at_block_pos"` kullanır. `mode: "coordinates"` signed decimal i32 `x`, `y`, `z`; `mode: "blockpos"` aynı fields; `mode: "packed"` 16 haneli unsigned hex `block_node` alır. `result.value.section_nodes`, callback sırasındaki packed SectionPos değerlerini unsigned lowercase hex olarak içerir.

Bu sözleşme callback side effect/laziness ve daha geniş streaming API davranışı yerine sonlu küçük node listesiyle sınırlıdır.

## Rust karşılığı

`crates/photon-core/src/binary.rs` içindeki `SectionPos::around_and_at_coordinates`, `around_and_at_block_pos` ve `around_and_at_packed`, Java overload yönlendirmesini, i32 wrapping `block±1` hesaplarını ve sectionX/sectionY/sectionZ callback sırasını taşır.


## Client-parity doğrulaması

`crates/photon-core/src/binary.rs`, SectionPos `aroundAndAtBlockPos` üç overload’unu Java ile aynı section node listesine ve sectionX/sectionY/sectionZ callback sırasına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos aroundAndAtBlockPos için 6 özgün senaryo hazırlandı.
- coordinates, BlockPos ve packed overload’ları; interior, sınır, köşe ve i32 taşması doğrudan Java/Rust API’leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.aroundAndAtBlockPos(BlockPos)`, packed long ve coordinate overload’larının küçük finite node listesi çıktısıyla sınırlıdır. `blocksInside`, range helpers, stream codec ve callback/laziness özellikleri ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
