# P01.c — SectionPos asLong overload'ları doğrulaması

`src/binary.rs`, SectionPos packed long üretimini packed BlockPos, doğrudan section koordinatları ve instance bileşenleri üzerinden Java ile aynı alan maskeleriyle yapar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos asLong overload'ları için 6 özgün senaryo hazırlandı.
- BlockPos section sınır geçişleri, pozitif/negatif 22/20 bit sınırları, keyfi packed girdiler ve i32 koordinat maskeleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1683/1683** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2185/2185** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `d91d4fb976af018c6ced8eb9d475396c09c64f7ef8429742661a9ba2d8c34267`. Birleşik referans kimliği: `595049f32d6822eff3379ec8f2d5211ef192043bf7f6a11db9b9d7c609fe01a1`.

Adapter hashleri: debug `a8905dba481a438cfebc22b452d1af22f2d3f1e7de4d2d3a376e85d6141eff61`, release `b6b7d1e7f5cd800ac6971e91b29dee4b36db61aef4e537adc1de1d2f5bff5f93`.

Yerel raporlar: `compare-arjeniq4` ve `compare-_0386q8q` (math debug/release), `compare-63qzbune` ve `compare-1pb7x57b` (birleşik debug/release), `verify-wl7nytn8` (math) ve `verify-f_mau559` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.asLong(BlockPos)`, `asLong(int, int, int)` ve instance `asLong()` yollarıyla sınırlıdır. SectionPos stream codec, `blocksInside` ve `aroundAndAtBlockPos` akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
