# P01.d — AABB.Builder doğrulaması

`src/aabb.rs`, Java `AABB.Builder` davranışını `AabbBuilder` ile uygular. `examples/behavior_adapter.rs`, f32 point bitlerini Rust API'sine taşır; Java oracle sonuçları doğrudan `AABB.Builder` çağrılarından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 995 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **995/995** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1497/1497** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda boş builder, tekli ve çoklu nokta birikimi, f32 hassasiyeti ve signed zero/NaN/sonsuzluk yolları karşılaştırıldı.
- Tanımsız `build()` sonucu Java'daki `IllegalStateException` yoluna karşı `undefined` sonucu olarak doğrulandı.
- `verify` çalıştırması kayıtlı 1497 senaryoluk Java referansını yeniden doğruladı.

Math referansı `f889b856cf0428a07f49c4e430a7c24ef0cd2f31ebfef1a99561960268d8e984`, birleşik referans `f43f8cbd1b5228b300d98e13e54dbf07d01ae1691a4a16ba16b2e05745e9b4e9` kimliğindedir.

Yerel raporlar: `compare-wzdfn9wg` (math debug), `compare-p0nj33p_` (math release), `compare-sdhltqtk` (birleşik debug), `compare-af9v4abw` (birleşik release) ve `verify-b_w9er_t` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik point dizisi ve bunların `isDefined()`/`build()` sonuçlarıyla sınırlıdır. `Vector3fc` için ayrı bir Rust vektör tipi eklenmedi; bileşenler f32 olarak alınır.
