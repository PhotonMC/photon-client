# P01.d — AABB.Builder doğrulaması

`src/aabb.rs`, Java `AABB.Builder` davranışını `AabbBuilder` ile uygular. `examples/behavior_adapter.rs`, f32 point bitlerini Rust API'sine taşır; Java oracle sonuçları doğrudan `AABB.Builder` çağrılarından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 887 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **887/887** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1341/1341** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda boş builder, tekli ve çoklu nokta birikimi, f32 hassasiyeti ve signed zero/NaN/sonsuzluk yolları karşılaştırıldı.
- Tanımsız `build()` sonucu Java'daki `IllegalStateException` yoluna karşı `undefined` sonucu olarak doğrulandı.
- `verify` çalıştırması kayıtlı 1341 senaryoluk Java referansını yeniden doğruladı.

Math referansı `676119520815e68dd9f4f120c8201463ce27aa4921910181223d0cdb38c939ec`, birleşik referans `971b4d305a95fb8e346f40a2edb704ac7f42751d32b8193786c4eb6aae63d1fb` kimliğindedir.

Yerel raporlar: `compare-d1bitq7h` (math debug), `compare-ldlepu8u` (math release), `compare-kuzkqdbl` (birleşik debug), `compare-n3x2kj9p` (birleşik release) ve `verify-jbdhw9mf` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik point dizisi ve bunların `isDefined()`/`build()` sonuçlarıyla sınırlıdır. `Vector3fc` için ayrı bir Rust vektör tipi eklenmedi; bileşenler f32 olarak alınır.
