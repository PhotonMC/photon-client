# P01.c — SectionPos packed dönüşümleri doğrulaması

`src/binary.rs`, packed BlockPos düğümlerini SectionPos düğümlerine çevirir, SectionPos y alanını temizler ve SectionPos x/z alanlarını ChunkPos packed düzenine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos packed dönüşümleri için 6 özgün senaryo hazırlandı.
- Origin, pozitif/negatif alanlar, 22/20 bit sınırları, i32 koordinat maskelenmesi ve keyfi bit desenleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1031/1031** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1533/1533** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `cbae17cd88ce5b1617d01034d27ad104db9b471eefc8bebf83ed4e8d1dc0aa70`. Birleşik referans kimliği: `338731e44c45c8acc6fbd70e7fe1439cdaa28ec5996f6a4d9517412cc50106a0`.

Adapter hashleri: debug `1e600a543381273dc98aed7417b4190eb8c9abf79baf0e9db83680822cd365ee`, release `d9e2c4949ccdd14729bb882019ab8eeebbfbd427228aadc4441ce6fb53e94a3d`.

Yerel raporlar: `compare-2ftkouxd` ve `compare-g204xxpd` (math debug/release), `compare-ag9x0lbh` ve `compare-sl2ed6ln` (birleşik debug/release), `verify-9ocfqh7` (math) ve `verify-5bpx38hs` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt packed SectionPos dönüşümleriyle sınırlıdır. SectionPos `asLong`, `offset` ve range yardımcıları dışındaki node akışları ile `blocksInside`/`aroundAndAtBlockPos` iterasyonları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
