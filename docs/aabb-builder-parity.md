# P01.d — AABB.Builder doğrulaması

`src/aabb.rs`, Java `AABB.Builder` davranışını `AabbBuilder` ile uygular. `examples/behavior_adapter.rs`, f32 point bitlerini Rust API'sine taşır; Java oracle sonuçları doğrudan `AABB.Builder` çağrılarından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 905 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **905/905** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1387/1387** senaryoda hem debug hem release adaptörüyle eşleşti.
- Beş yeni senaryoda boş builder, tekli ve çoklu nokta birikimi, f32 hassasiyeti ve signed zero/NaN/sonsuzluk yolları karşılaştırıldı.
- Tanımsız `build()` sonucu Java'daki `IllegalStateException` yoluna karşı `undefined` sonucu olarak doğrulandı.
- `verify` çalıştırması kayıtlı 1387 senaryoluk Java referansını yeniden doğruladı.

Math referansı `847816d7bf7d2e0e9cecc67b5d7010685842d00cd4860fb3cf9447d37a9fe8a9`, birleşik referans `a2de3a61075984570c78972cf64abf406bbb6ee6d60136396680e14b82219664` kimliğindedir.

Yerel raporlar: `compare-psz_s0y8` (math debug), `compare-xhpjkx1_` (math release), `compare-1i6_miir` (birleşik debug), `compare-zyhj5dym` (birleşik release) ve `verify-b04aqmvb` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt, beş özgün sentetik point dizisi ve bunların `isDefined()`/`build()` sonuçlarıyla sınırlıdır. `Vector3fc` için ayrı bir Rust vektör tipi eklenmedi; bileşenler f32 olarak alınır.
