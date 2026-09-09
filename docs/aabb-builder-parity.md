# P01.d — AABB.Builder doğrulaması

`src/aabb.rs`, Java `AABB.Builder` davranışını `AabbBuilder` ile uygular. `examples/behavior_adapter.rs`, f32 point bitlerini Rust API'sine taşır; Java oracle sonuçları doğrudan `AABB.Builder` çağrılarından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 660 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **660/660** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1114/1114** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda boş builder, tekli ve çoklu nokta birikimi, f32 hassasiyeti ve signed zero/NaN/sonsuzluk yolları karşılaştırıldı.
- Tanımsız `build()` sonucu Java'daki `IllegalStateException` yoluna karşı `undefined` sonucu olarak doğrulandı.
- `verify` çalıştırması kayıtlı 1114 senaryoluk Java referansını yeniden doğruladı.

Math referansı `bc87ac5ece4a47cf40d4e562d69ac72206437e2c117cd058a79561ba4b4d149b`, birleşik referans `15dc52b0af0addc9b446d43666846404d2d8e35a7cc7751489632ec703b826b8` kimliğindedir.

Yerel raporlar: `compare-3kg8m9ta` (math debug), `compare-6crwfl2x` (math release), `compare-_35t8r3l` (birleşik debug), `compare-g5oda9d_` (birleşik release) ve `verify-6xepcluv` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik point dizisi ve bunların `isDefined()`/`build()` sonuçlarıyla sınırlıdır. `Vector3fc` için ayrı bir Rust vektör tipi eklenmedi; bileşenler f32 olarak alınır.
