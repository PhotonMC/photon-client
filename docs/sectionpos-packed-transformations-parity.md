# P01.c — SectionPos packed dönüşümleri doğrulaması

`src/binary.rs`, packed BlockPos düğümlerini SectionPos düğümlerine çevirir, SectionPos y alanını temizler ve SectionPos x/z alanlarını ChunkPos packed düzenine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed dönüşümleri için 6 özgün senaryo hazırlandı.
- Origin, pozitif/negatif alanlar, 22/20 bit sınırları, i32 koordinat maskelenmesi ve keyfi bit desenleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1354/1354** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1856/1856** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `7bacfa4674e388b5f38f1b0a041e5531afe27e845d4374895c79e4fb56697a24`. Birleşik referans kimliği: `3afb23055305f62e13f7bdef0f8d340c8294187f00de3b26a2c52874e45642a7`.

Adapter hashleri: debug `45dccbceba09502c7c6c8e18ca105a628633653b7a27d8cb49b0aba1229bac62`, release `8274db44ba308fe51298d304e3f2dfbd62e8e2a353a0f84b806ee6d3643b26db`.

Yerel raporlar: `compare-z9ypl4_s` ve `compare-0xhgd0yz` (math debug/release), `compare-ote21q3t` ve `compare-r5nijsku` (birleşik debug/release), `verify-33997f0f` (math) ve `verify-2b2qykhl` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt packed SectionPos dönüşümleriyle sınırlıdır. SectionPos `asLong`, `offset` ve range yardımcıları dışındaki node akışları ile `blocksInside`/`aroundAndAtBlockPos` iterasyonları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
