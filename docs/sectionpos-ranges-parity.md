# P01.c — SectionPos range yardımcıları doğrulaması

`src/binary.rs`, SectionPos `betweenClosedStream`, `cube` ve `aroundChunk` traversal'larını Java ile aynı inclusive x/y/z sırasına ve packed sonuçlara bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos range yardımcıları için 6 özgün senaryo hazırlandı.
- Üç static yolun tek nokta, negatif koordinat, cube, i32 sınırı ve around-chunk dikey aralık davranışları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1046/1046** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1548/1548** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `bd99eadfa488c44d624d410a441fe8b99d4485c7236dd276609af1edc3394f3b`. Birleşik referans kimliği: `7a79765a9b449a8ea9c09f049a1ab917958fff879c31b1058ce2f4b8dd3208ab`.

Adapter hashleri: debug `be5945a45013303bd4a3b00ae602af8bd5d5fe8e4e74b4de87fbe87078de62d5`, release `62c4febc034bc6e788ad25444d63fce5be8a5f781a5394db9369e01efae8f635`.

Yerel raporlar: `compare-m8v98wom` ve `compare-4i19c_i3` (math debug/release), `compare-p7m9abjy` ve `compare-vvlbkmvf` (birleşik debug/release), `verify-92kir_up` (math) ve `verify-_w4izisu` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.betweenClosedStream`, `cube` ve `aroundChunk` static yollarının sonlu, pozitif cursor boyutlu çıktısıyla sınırlıdır. `blocksInside`, `aroundAndAtBlockPos`, SectionPos stream codec'i, stream laziness/spliterator özellikleri ve overflow ile devasa aralıklar ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
