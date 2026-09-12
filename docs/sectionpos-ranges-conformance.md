# P01.c — SectionPos range yardımcıları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, SectionPos'ın inclusive `betweenClosedStream`, `cube` ve `aroundChunk` yollarının sonlu çıktı sırasıdır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `.data/minecraft/source/net/minecraft/core/SectionPos.java:234-264` ve `.data/minecraft/source/net/minecraft/core/Cursor3D.java:8-45` incelendi. Oracle Java stream'lerini doğrudan tüketir; beklenen sonuçlar Rust çıktısından türetilmez.

`betweenClosedStream`, min ve max uçlarını dahil eder. Cursor, x eksenini hızlı değiştirir; x satırı tamamlanınca y, y dilimi tamamlanınca z ilerler. `cube`, merkez section'ın üç bileşeninden radius çıkarıp ekleyerek bu akışa bağlanır. `aroundChunk`, ChunkPos'ın x/z bileşenlerinden yatay uçları ve verilen min/max section değerlerinden dikey uçları kurar.

## Senaryolar ve JSONL

| Grup | Sayı | Kapsam |
| --- | ---: | --- |
| `betweenClosedStream` | 3 | Tek nokta, x/y/z sırası ve negatif koordinatlar |
| `cube` | 2 | Origin çevresinde radius ve i32 sınır değerli tek section |
| `aroundChunk` | 1 | Yatay radius ve iki section yüksekliği |

Girdiler `family: "math"`, `operation: "sectionpos_ranges"` ve `mode` alanını kullanır. `between` modu min/max x/y/z; `cube` modu center x/y/z ve radius; `around-chunk` modu chunk x/z, radius ve min/max section alanlarını taşır. `result.value.positions`, x/y/z koordinatlarını ve `asLong()` packed değerini Java traversal sırasıyla içerir.

Bu sözleşme sonlu ve pozitif Cursor3D boyutlarına sahip aralıkları ölçer. Java stream tembelliği, spliterator estimate değeri, paralel tüketim ve taşarak devasa veya geçersiz aralık oluşturan i32 çarpımları bu dilime dahil değildir.

## Rust karşılığı

`src/binary.rs` içindeki `SectionPos::between_closed`, `cube` ve `around_chunk`, Java'nın x-fastest/y-next/z-next sırasını ve merkez uçlarının i32 wrapping aritmetiğini üretir. `examples/behavior_adapter.rs` üç yolu JSONL operation'ına bağlar; `tests/behavior/java/MathOracle.java` Java stream'lerini karşılaştırma oracle'ı olarak tüketir.


## Client-parity doğrulaması

`src/binary.rs`, SectionPos `betweenClosedStream`, `cube` ve `aroundChunk` traversal'larını Java ile aynı inclusive x/y/z sırasına ve packed sonuçlara bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos range yardımcıları için 6 özgün senaryo hazırlandı.
- Üç static yolun tek nokta, negatif koordinat, cube, i32 sınırı ve around-chunk dikey aralık davranışları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.betweenClosedStream`, `cube` ve `aroundChunk` static yollarının sonlu, pozitif cursor boyutlu çıktısıyla sınırlıdır. `blocksInside`, `aroundAndAtBlockPos`, SectionPos stream codec'i, stream laziness/spliterator özellikleri ve overflow ile devasa aralıklar ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
