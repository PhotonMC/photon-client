# P01.d — AABB.Builder doğrulaması

`src/aabb.rs`, Java `AABB.Builder` davranışını `AabbBuilder` ile uygular. `examples/behavior_adapter.rs`, f32 point bitlerini Rust API'sine taşır; Java oracle sonuçları doğrudan `AABB.Builder` çağrılarından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1025 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1025/1025** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1527/1527** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda boş builder, tekli ve çoklu nokta birikimi, f32 hassasiyeti ve signed zero/NaN/sonsuzluk yolları karşılaştırıldı.
- Tanımsız `build()` sonucu Java'daki `IllegalStateException` yoluna karşı `undefined` sonucu olarak doğrulandı.
- `verify` çalıştırması kayıtlı 1527 senaryoluk Java referansını yeniden doğruladı.

Math referansı `2c79e2aaba1154782177df83cd39488c1c9a365c76e1971733e837c447fe9fa3`, birleşik referans `74f88dfa13fb755a92b838953181016d177ef3a05b113b5fa8ef1f3735031acc` kimliğindedir.

Yerel raporlar: `compare-0dsoh6yn` (math debug), `compare-wdh5yokb` (math release), `compare-7wace8y9` (birleşik debug), `compare-ohni39xi` (birleşik release) ve `verify-a7iqdyvs` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik point dizisi ve bunların `isDefined()`/`build()` sonuçlarıyla sınırlıdır. `Vector3fc` için ayrı bir Rust vektör tipi eklenmedi; bileşenler f32 olarak alınır.
