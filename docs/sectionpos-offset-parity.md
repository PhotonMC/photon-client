# P01.c — SectionPos offset overload'ları doğrulaması

`src/binary.rs`, SectionPos packed düğümlerini Direction veya üç adımla kaydırır ve instance koordinatlarını Java ile aynı i32 toplama ve packed field-mask yollarıyla üretir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos offset overload'ları için 6 özgün senaryo hazırlandı.
- Direction/static/instance çağrı yolları, sıfır adım, pozitif/negatif değerler, packed alan sınırı ve i32 taşması doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1703/1703** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2205/2205** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `b93cda4c5257c03011378a24fe916e8b8cd7e11d2fe75e1a1de108e4b4ebd630`. Birleşik referans kimliği: `413b981c2a619373545c6c4c74d3e7fec6b1a2e1dc34d5a49ee2f4eed5231162`.

Adapter hashleri: debug `c1464c3995fc085081d8d7d5957a9eca938185a498315c143c30d63e829ffe90`, release `a68437319cbfe090fc182a3c8ec431a6c95605056e456fc47471c96ea253ffda`.

Yerel raporlar: `compare-a8qhmnrl` ve `compare-usm27y8z` (math debug/release), `compare-8vwtvwwl` ve `compare-uey51w68` (birleşik debug/release), `verify-d3ywhdwp` (math) ve `verify-l3irw60r` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.offset(long, Direction)`, `offset(long, int, int, int)` ve instance `offset(int, int, int)` yollarıyla sınırlıdır. SectionPos stream codec, `blocksInside` ve `aroundAndAtBlockPos` ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
