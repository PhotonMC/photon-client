# P01.b — Vec2 add overload'ları doğrulaması

`src/math.rs`, Java `Vec2.add(Vec2)` ve `Vec2.add(float)` metodlarını karşılayan `Vec2::add_vec2` ve `Vec2::add_scalar` metodlarını içerir. Java oracle doğrudan `add` çağrılarını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1040 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1040/1040** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec2 ve scalar overload'ları, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1542/1542** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1542 senaryoluk Java referansını yeniden doğruladı.

Math referansı `a21163be0f5e47c3d99a4468d2f3b67c6d04d0dcee95ca6b70f49b581fac358c`, birleşik referans `b28d88fa06544e5713186c8c0c44bacbc4c7be8b946c7932a025574a1b3b2126` kimliklerindedir.

Yerel raporlar: `compare-_rqqaqfb` (math debug), `compare-v1ltm6cc` (math release), `compare-1lz8q4ar` (birleşik debug), `compare-fvdcbn0r` (birleşik release) ve `verify-nud8oh4e` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 girdisi ve add sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 dot, rotate, distance ve diğer yardımcıları ayrı davranış dilimleridir.
