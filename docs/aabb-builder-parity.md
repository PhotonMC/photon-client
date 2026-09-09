# P01.d — AABB.Builder doğrulaması

`src/aabb.rs`, Java `AABB.Builder` davranışını `AabbBuilder` ile uygular. `examples/behavior_adapter.rs`, f32 point bitlerini Rust API'sine taşır; Java oracle sonuçları doğrudan `AABB.Builder` çağrılarından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1149 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1149/1149** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1651/1651** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda boş builder, tekli ve çoklu nokta birikimi, f32 hassasiyeti ve signed zero/NaN/sonsuzluk yolları karşılaştırıldı.
- Tanımsız `build()` sonucu Java'daki `IllegalStateException` yoluna karşı `undefined` sonucu olarak doğrulandı.
- `verify` çalıştırması kayıtlı 1651 senaryoluk Java referansını yeniden doğruladı.

Math referansı `2f973f986814298b094b982753eef5a170865c5228762b233a6060766b5ed32e`, birleşik referans `f8cfd3cf28e07e5859a339a4844b50e474cb8351ea003bb36ba96ccaf38abec1` kimliğindedir.

Yerel raporlar: `compare-hz521aor` (math debug), `compare-uded0zec` (math release), `compare-nc1496mg` (birleşik debug), `compare-ljx3yrp0` (birleşik release) ve `verify-k950tc5l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik point dizisi ve bunların `isDefined()`/`build()` sonuçlarıyla sınırlıdır. `Vector3fc` için ayrı bir Rust vektör tipi eklenmedi; bileşenler f32 olarak alınır.
