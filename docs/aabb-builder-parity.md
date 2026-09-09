# P01.d — AABB.Builder doğrulaması

`src/aabb.rs`, Java `AABB.Builder` davranışını `AabbBuilder` ile uygular. `examples/behavior_adapter.rs`, f32 point bitlerini Rust API'sine taşır; Java oracle sonuçları doğrudan `AABB.Builder` çağrılarından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 700 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **700/700** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1154/1154** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda boş builder, tekli ve çoklu nokta birikimi, f32 hassasiyeti ve signed zero/NaN/sonsuzluk yolları karşılaştırıldı.
- Tanımsız `build()` sonucu Java'daki `IllegalStateException` yoluna karşı `undefined` sonucu olarak doğrulandı.
- `verify` çalıştırması kayıtlı 1154 senaryoluk Java referansını yeniden doğruladı.

Math referansı `c1800d2e244360700681ce9dd25627eeb76c5b378b61d6c6a9894994bc858ad8`, birleşik referans `040454adf30b9e5c2e8c273080741126bc70be6ae1b1c6d5176978231e9d036a` kimliğindedir.

Yerel raporlar: `compare-yy9k89p5` (math debug), `compare-og6yrygd` (math release), `compare-downo9zd` (birleşik debug), `compare-66zm55sg` (birleşik release) ve `verify-tf9y7j96` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik point dizisi ve bunların `isDefined()`/`build()` sonuçlarıyla sınırlıdır. `Vector3fc` için ayrı bir Rust vektör tipi eklenmedi; bileşenler f32 olarak alınır.
