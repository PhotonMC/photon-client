# P01.c — SectionPos offset overload'ları doğrulaması

`src/binary.rs`, SectionPos packed düğümlerini Direction veya üç adımla kaydırır ve instance koordinatlarını Java ile aynı i32 toplama ve packed field-mask yollarıyla üretir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos offset overload'ları için 6 özgün senaryo hazırlandı.
- Direction/static/instance çağrı yolları, sıfır adım, pozitif/negatif değerler, packed alan sınırı ve i32 taşması doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1653/1653** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2155/2155** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `e9b1c1e55b2a03528b712cd304101c7b36378556372ebad60bb330a4a60eb192`. Birleşik referans kimliği: `2a68356f797e134e8a268355635fffb2db6a83b94e11d3afbdf308978093028e`.

Adapter hashleri: debug `2371e57dd5b14642fa55352a223485b414fbdeb72599f35405f7e4517da68372`, release `ce1f002fb70be2fce7ebe09258c7d8ffbc7eed67c9643ef81eeb6da2dabb4ecb`.

Yerel raporlar: `compare-ceusm4y4` ve `compare-hconpy1a` (math debug/release), `compare-rhjx0z7e` ve `compare-jtl411eu` (birleşik debug/release), `verify-biycqhmq` (math) ve `verify-u5d5zd4c` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.offset(long, Direction)`, `offset(long, int, int, int)` ve instance `offset(int, int, int)` yollarıyla sınırlıdır. SectionPos stream codec, `blocksInside` ve `aroundAndAtBlockPos` ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
