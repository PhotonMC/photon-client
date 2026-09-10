# P01.c — SectionPos range yardımcıları doğrulaması

`src/binary.rs`, SectionPos `betweenClosedStream`, `cube` ve `aroundChunk` traversal'larını Java ile aynı inclusive x/y/z sırasına ve packed sonuçlara bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos range yardımcıları için 6 özgün senaryo hazırlandı.
- Üç static yolun tek nokta, negatif koordinat, cube, i32 sınırı ve around-chunk dikey aralık davranışları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1843/1843** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2345/2345** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `dfeb55b1b61cbc7649df7ae8af88469cbf7480a194892aacbcf8e5e5b93806bd`. Birleşik referans kimliği: `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576`.

Adapter hashleri: debug `c3f51b7830e3c0554138ff428a979a8f54ffaf85f7e2082bc6962bdcf01ff1d9`, release `fa350ee97911ed79b9b6db86a8af51e8b7dc87401e8312957b94741266b4b51b`.

Yerel raporlar: `compare-5_avciij` ve `compare-e6e6agr4` (math debug/release), `compare-gvo3eind` ve `compare-wmabbl5t` (birleşik debug/release), `verify-zbtpetmh` (math) ve `verify-vzenwlxd` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.betweenClosedStream`, `cube` ve `aroundChunk` static yollarının sonlu, pozitif cursor boyutlu çıktısıyla sınırlıdır. `blocksInside`, `aroundAndAtBlockPos`, SectionPos stream codec'i, stream laziness/spliterator özellikleri ve overflow ile devasa aralıklar ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
