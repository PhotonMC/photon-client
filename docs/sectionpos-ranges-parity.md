# P01.c — SectionPos range yardımcıları doğrulaması

`src/binary.rs`, SectionPos `betweenClosedStream`, `cube` ve `aroundChunk` traversal'larını Java ile aynı inclusive x/y/z sırasına ve packed sonuçlara bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos range yardımcıları için 6 özgün senaryo hazırlandı.
- Üç static yolun tek nokta, negatif koordinat, cube, i32 sınırı ve around-chunk dikey aralık davranışları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1987/1987** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2489/2489** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `665bdda6827e71f559f4e9a959ed126f3d3e055594f3b677359c51d0d3a74e16`. Birleşik referans kimliği: `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc`.

Adapter hashleri: debug `bf472fbe984c9033c542831c832c85f3ea36283e022040542677049db4615a06`, release `04de447e6d530b9e66134316577c0e326f349761b64ee41c18f4e76fd6d3b226`.

Yerel raporlar: `compare-zt_h_egg` ve `compare-7afcvx21` (math debug/release), `compare-251aed0_` ve `compare-jqj2kta_` (birleşik debug/release), `verify-5v_ilol4` (math) ve `verify-csdh4b83` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.betweenClosedStream`, `cube` ve `aroundChunk` static yollarının sonlu, pozitif cursor boyutlu çıktısıyla sınırlıdır. `blocksInside`, `aroundAndAtBlockPos`, SectionPos stream codec'i, stream laziness/spliterator özellikleri ve overflow ile devasa aralıklar ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
