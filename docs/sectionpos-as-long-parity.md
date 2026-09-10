# P01.c — SectionPos asLong overload'ları doğrulaması

`src/binary.rs`, SectionPos packed long üretimini packed BlockPos, doğrudan section koordinatları ve instance bileşenleri üzerinden Java ile aynı alan maskeleriyle yapar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos asLong overload'ları için 6 özgün senaryo hazırlandı.
- BlockPos section sınır geçişleri, pozitif/negatif 22/20 bit sınırları, keyfi packed girdiler ve i32 koordinat maskeleri doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2076/2076** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2578/2578** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `e84d285a8cd335bd1ba5cd88d076e7cb38315013b991057804c6f1b4598f1c80`. Birleşik referans kimliği: `9c94496c39504f2bd146d331630516f020e05259c67c0bca659d1f1ca92c74a8`.

Adapter hashleri: debug `6eb1e772a2e06c20b8d0c09ba0b2a11b5f1df7a671ee8d1513127bc318d143ef`, release `318c5ef22a8d56277aa19f9da9f5becca9f575904eba79756b959bdc8c993fe88`.

Yerel raporlar: `compare-k2v65l83` ve `compare-9uf1gox2` (math debug/release), `compare-ptlm9ptr` ve `compare-86z18x_k` (birleşik debug/release), `verify-9dtf6rea` (math) ve `verify-31k8bjcu` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `SectionPos.asLong(BlockPos)`, `asLong(int, int, int)` ve instance `asLong()` yollarıyla sınırlıdır. SectionPos stream codec, `blocksInside` ve `aroundAndAtBlockPos` akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
