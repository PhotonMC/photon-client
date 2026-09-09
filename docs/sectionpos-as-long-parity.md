# P01.c — SectionPos asLong overload'ları doğrulaması

`src/binary.rs`, SectionPos packed long üretimini packed BlockPos, doğrudan section koordinatları ve instance bileşenleri üzerinden Java ile aynı alan maskeleriyle yapar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos asLong overload'ları için 6 özgün senaryo hazırlandı.
- BlockPos section sınır geçişleri, pozitif/negatif 22/20 bit sınırları, keyfi packed girdiler ve i32 koordinat maskeleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1067/1067** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1569/1569** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `516108a2035e89e9716bb1dcd67352eaa358d73159f8e8391e2d51372d77d713`. Birleşik referans kimliği: `9f76d4ed1ee97b0d89ef969b0fdf139f7e658f9316895af88b889f790f13b8d1`.

Adapter hashleri: debug `2e569fd5b282fc3c6202cf4a505416139c9e9cf90f6ebc711c0851970da8bc15`, release `72e7c71370f232b44263e273b357aab840c978acec80458d8e02a7b31aef41a1`.

Yerel raporlar: `compare-d3vh9fy0` ve `compare-cdl_uz9p` (math debug/release), `compare-b4u1_5xx` ve `compare-lz2txc82` (birleşik debug/release), `verify-cmxttwgw` (math) ve `verify-i4bptixz` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.asLong(BlockPos)`, `asLong(int, int, int)` ve instance `asLong()` yollarıyla sınırlıdır. SectionPos stream codec, `blocksInside` ve `aroundAndAtBlockPos` akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
