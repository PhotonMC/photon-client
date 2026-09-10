# P01.c — SectionPos aroundAndAtBlockPos doğrulaması

`src/binary.rs`, SectionPos `aroundAndAtBlockPos` üç overload’unu Java ile aynı section node listesine ve sectionX/sectionY/sectionZ callback sırasına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos aroundAndAtBlockPos için 6 özgün senaryo hazırlandı.
- coordinates, BlockPos ve packed overload’ları; interior, sınır, köşe ve i32 taşması doğrudan Java/Rust API’leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1422/1422** math senaryosunda eşleşti.
- Birleşik altı aile corpus’u **1924/1924** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `cc96d24cc80c682020b8b8d28795f7b434dad5677010e1608fc1b6e2a730e5d7`. Birleşik referans kimliği: `a346583dc8bd7a5d672fe737c86d9febc8a2861cc99d0c486416f41813870b7c`.

Adapter hashleri: debug `4e30afd61f45820df0f57541587fa8df456d9a1374ec71795c01f465ec26de87`, release `73c1c9ea2bef7322d279fc4f84d077a2009745e23f36ce680884b73fe11b7d45`.

Yerel raporlar: `compare-ox0b8e23` (math debug), `compare-8yyrdrg5` (math release), `compare-xbim8rgs` (birleşik debug), `compare-kg978z18` (birleşik release), `verify-p7_v_2m3` (math) ve `verify-tw8nqf_h` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.aroundAndAtBlockPos(BlockPos)`, packed long ve coordinate overload’larının küçük finite node listesi çıktısıyla sınırlıdır. `blocksInside`, range helpers, stream codec ve callback/laziness özellikleri ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
