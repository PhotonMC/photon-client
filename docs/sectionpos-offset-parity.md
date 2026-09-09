# P01.c — SectionPos offset overload'ları doğrulaması

`src/binary.rs`, SectionPos packed düğümlerini Direction veya üç adımla kaydırır ve instance koordinatlarını Java ile aynı i32 toplama ve packed field-mask yollarıyla üretir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos offset overload'ları için 6 özgün senaryo hazırlandı.
- Direction/static/instance çağrı yolları, sıfır adım, pozitif/negatif değerler, packed alan sınırı ve i32 taşması doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1052/1052** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1554/1554** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `d4f17c1024ea8a511a4611971e222dcdaa1a00ec6c1baa442420a0f051c914cc`. Birleşik referans kimliği: `f615eef08df36fc7705e9f9da886cfc498964867c630a79cb4bf7f15f3c0bae1`.

Adapter hashleri: debug `bbf24cf089b0f2cd95ea3c80c126162162c7c68897a17f143d806373787cf1ab`, release `2e01f137cfb935c1ff43cab5c31ca3c1a8f4aa1dd10f2b1afb45d13d768aaccd`.

Yerel raporlar: `compare-xj4m01k2` ve `compare-ix4n2052` (math debug/release), `compare-e0lm9mqh` ve `compare-pp8moc03` (birleşik debug/release), `verify-s0vlo43l` (math) ve `verify-r4qtgn3b` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.offset(long, Direction)`, `offset(long, int, int, int)` ve instance `offset(int, int, int)` yollarıyla sınırlıdır. SectionPos stream codec, `blocksInside` ve `aroundAndAtBlockPos` ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
