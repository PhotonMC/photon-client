# P01.c — SectionPos asLong overload'ları doğrulaması

`src/binary.rs`, SectionPos packed long üretimini packed BlockPos, doğrudan section koordinatları ve instance bileşenleri üzerinden Java ile aynı alan maskeleriyle yapar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos asLong overload'ları için 6 özgün senaryo hazırlandı.
- BlockPos section sınır geçişleri, pozitif/negatif 22/20 bit sınırları, keyfi packed girdiler ve i32 koordinat maskeleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1422/1422** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1924/1924** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `cc96d24cc80c682020b8b8d28795f7b434dad5677010e1608fc1b6e2a730e5d7`. Birleşik referans kimliği: `a346583dc8bd7a5d672fe737c86d9febc8a2861cc99d0c486416f41813870b7c`.

Adapter hashleri: debug `4e30afd61f45820df0f57541587fa8df456d9a1374ec71795c01f465ec26de87`, release `73c1c9ea2bef7322d279fc4f84d077a2009745e23f36ce680884b73fe11b7d45`.

Yerel raporlar: `compare-ox0b8e23` ve `compare-8yyrdrg5` (math debug/release), `compare-xbim8rgs` ve `compare-kg978z18` (birleşik debug/release), `verify-p7_v_2m3` (math) ve `verify-tw8nqf_h` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.asLong(BlockPos)`, `asLong(int, int, int)` ve instance `asLong()` yollarıyla sınırlıdır. SectionPos stream codec, `blocksInside` ve `aroundAndAtBlockPos` akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
