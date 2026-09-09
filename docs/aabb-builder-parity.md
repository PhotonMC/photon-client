# P01.d — AABB.Builder doğrulaması

`src/aabb.rs`, Java `AABB.Builder` davranışını `AabbBuilder` ile uygular. `examples/behavior_adapter.rs`, f32 point bitlerini Rust API'sine taşır; Java oracle sonuçları doğrudan `AABB.Builder` çağrılarından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 777 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **777/777** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1231/1231** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda boş builder, tekli ve çoklu nokta birikimi, f32 hassasiyeti ve signed zero/NaN/sonsuzluk yolları karşılaştırıldı.
- Tanımsız `build()` sonucu Java'daki `IllegalStateException` yoluna karşı `undefined` sonucu olarak doğrulandı.
- `verify` çalıştırması kayıtlı 1231 senaryoluk Java referansını yeniden doğruladı.

Math referansı `d6723874f1f8073346a0f729f2c06b2e00e2dcd34baad1a33733698d293350d3`, birleşik referans `0f128e8e7fb68abc882891752c85767e02d3b02a6b9f83fff7f87b45ab6f2afe` kimliğindedir.

Yerel raporlar: `compare-da6k05zu` (math debug), `compare-jem8yqw3` (math release), `compare-r10lr0oe` (birleşik debug), `compare-rb0k9jx5` (birleşik release) ve `verify-04cazxan` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik point dizisi ve bunların `isDefined()`/`build()` sonuçlarıyla sınırlıdır. `Vector3fc` için ayrı bir Rust vektör tipi eklenmedi; bileşenler f32 olarak alınır.
