# P01.c — SectionPos asLong overload'ları doğrulaması

`src/binary.rs`, SectionPos packed long üretimini packed BlockPos, doğrudan section koordinatları ve instance bileşenleri üzerinden Java ile aynı alan maskeleriyle yapar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos asLong overload'ları için 6 özgün senaryo hazırlandı.
- BlockPos section sınır geçişleri, pozitif/negatif 22/20 bit sınırları, keyfi packed girdiler ve i32 koordinat maskeleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1019/1019** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1521/1521** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `5df2c7aa290a2d3deb2e318553a41a27dbe6b08b22e1d31b68362eedc36e2108`. Birleşik referans kimliği: `53cb6b5a4397567e26419e7466bcd8577c97308e626648d85f2a40bb55fb2406`.

Adapter hashleri: debug `16d22ebaba2149325070f38dc9f824049f2dabd75096e5e62690f4382900cdf2`, release `b5946ae548e5415df7ce5856927d83f5b454c350ac8d8ef6164949b8a7eca5f1`.

Yerel raporlar: `compare-_lwapdio` ve `compare-ed7p25g6` (math debug/release), `compare-uh_quimp` ve `compare-233_yr3a` (birleşik debug/release), `verify-9lc8nch` (math) ve `verify-wqacm20n` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.asLong(BlockPos)`, `asLong(int, int, int)` ve instance `asLong()` yollarıyla sınırlıdır. SectionPos `offset`, stream codec ve `blocksInside`/`cube` iterasyonları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
