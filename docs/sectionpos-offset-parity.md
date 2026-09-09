# P01.c — SectionPos offset overload'ları doğrulaması

`src/binary.rs`, SectionPos packed düğümlerini Direction veya üç adımla kaydırır ve instance koordinatlarını Java ile aynı i32 toplama ve packed field-mask yollarıyla üretir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos offset overload'ları için 6 özgün senaryo hazırlandı.
- Direction/static/instance çağrı yolları, sıfır adım, pozitif/negatif değerler, packed alan sınırı ve i32 taşması doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1092/1092** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1594/1594** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `8475d3304542000c5493cb030a2dfb280baa6a64c19a4ba3eb09d75f9c0e67a8`. Birleşik referans kimliği: `a15082ba584c6d0bca4bc52d80fd5da8fc49f05193cec75afd71cf0af0c4ee0d`.

Adapter hashleri: debug `61f28350754db6b9e7c92d36856a1f48106a81b9772704394da4169de4cdc47b`, release `12a5b4b38b7716b91145f537aac7395892c11bdfca6c18002fa0094da7ca7816`.

Yerel raporlar: `compare-ta5w_83f` ve `compare-vzkk24q9` (math debug/release), `compare-271dl4fv` ve `compare-y8i46uni` (birleşik debug/release), `verify-0qcp4kkh` (math) ve `verify-3t751q44` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.offset(long, Direction)`, `offset(long, int, int, int)` ve instance `offset(int, int, int)` yollarıyla sınırlıdır. SectionPos stream codec, `blocksInside` ve `aroundAndAtBlockPos` ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
