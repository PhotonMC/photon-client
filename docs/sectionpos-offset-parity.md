# P01.c — SectionPos offset overload'ları doğrulaması

`src/binary.rs`, SectionPos packed düğümlerini Direction veya üç adımla kaydırır ve instance koordinatlarını Java ile aynı i32 toplama ve packed field-mask yollarıyla üretir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos offset overload'ları için 6 özgün senaryo hazırlandı.
- Direction/static/instance çağrı yolları, sıfır adım, pozitif/negatif değerler, packed alan sınırı ve i32 taşması doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1319/1319** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1821/1821** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `85b9ce09336ec312c84a062e772cdb41b6a2e079b6fcb1995200470edf61dd6c`. Birleşik referans kimliği: `01a770901948bb0f5650c8b4e80bf54b513d904c9746232c38b93c03a7546957`.

Adapter hashleri: debug `43e1243cabb026db4daeb5b4e9f959cceeea3e253000b2c7e57159bade72d306`, release `01aaad8ffc0dccd9e142dee68c1e26f58b2554506e1fc2750ea7092e4f50fe1b`.

Yerel raporlar: `compare-stkj42d2` ve `compare-e9upg098` (math debug/release), `compare-olmqgxpc` ve `compare-jxg0__tj` (birleşik debug/release), `verify-6wifh_w5` (math) ve `verify-m0natom3` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.offset(long, Direction)`, `offset(long, int, int, int)` ve instance `offset(int, int, int)` yollarıyla sınırlıdır. SectionPos stream codec, `blocksInside` ve `aroundAndAtBlockPos` ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
