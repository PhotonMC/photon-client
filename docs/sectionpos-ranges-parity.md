# P01.c — SectionPos range yardımcıları doğrulaması

`src/binary.rs`, SectionPos `betweenClosedStream`, `cube` ve `aroundChunk` traversal'larını Java ile aynı inclusive x/y/z sırasına ve packed sonuçlara bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos range yardımcıları için 6 özgün senaryo hazırlandı.
- Üç static yolun tek nokta, negatif koordinat, cube, i32 sınırı ve around-chunk dikey aralık davranışları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1999/1999** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2501/2501** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `d0cf0bb203c8ea48ad8144fae6a58022846d1a5cfb95b398018620bae7b0bc3d`. Birleşik referans kimliği: `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040`.

Adapter hashleri: debug `5d019ee46fe3c7981d2881f496667dfc50b632b55a72f1c124178a69528f7705`, release `6081f381a8d8391d22750c81b039167dc2cb4dabf8274f6410be0463eb1881a8`.

Yerel raporlar: `compare-druvbg3h` ve `compare-k2ed8y70` (math debug/release), `compare-h1kbcaxz` ve `compare-js7_mas4` (birleşik debug/release), `verify-izjnelsu` (math) ve `verify-d2ptfzig` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.betweenClosedStream`, `cube` ve `aroundChunk` static yollarının sonlu, pozitif cursor boyutlu çıktısıyla sınırlıdır. `blocksInside`, `aroundAndAtBlockPos`, SectionPos stream codec'i, stream laziness/spliterator özellikleri ve overflow ile devasa aralıklar ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
