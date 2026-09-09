# P01.d — AABB.Builder doğrulaması

`src/aabb.rs`, Java `AABB.Builder` davranışını `AabbBuilder` ile uygular. `examples/behavior_adapter.rs`, f32 point bitlerini Rust API'sine taşır; Java oracle sonuçları doğrudan `AABB.Builder` çağrılarından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 665 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **665/665** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1119/1119** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda boş builder, tekli ve çoklu nokta birikimi, f32 hassasiyeti ve signed zero/NaN/sonsuzluk yolları karşılaştırıldı.
- Tanımsız `build()` sonucu Java'daki `IllegalStateException` yoluna karşı `undefined` sonucu olarak doğrulandı.
- `verify` çalıştırması kayıtlı 1119 senaryoluk Java referansını yeniden doğruladı.

Math referansı `b4f21f456b1ed4b8f6d8d04534a21132ed8c550c6cc3e8f152c696a895bf9de0`, birleşik referans `3b2bdc05bfdf0766c7665b84746689cdc5d05a2b2c751cf2d9a7ceaf66690145` kimliğindedir.

Yerel raporlar: `compare-2pd_izg6` (math debug), `compare-k81tuucm` (math release), `compare-ybb_vjnz` (birleşik debug), `compare-xxko56va` (birleşik release) ve `verify-p5bur1pm` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik point dizisi ve bunların `isDefined()`/`build()` sonuçlarıyla sınırlıdır. `Vector3fc` için ayrı bir Rust vektör tipi eklenmedi; bileşenler f32 olarak alınır.
