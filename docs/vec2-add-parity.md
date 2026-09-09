# P01.b — Vec2 add overload'ları doğrulaması

`src/math.rs`, Java `Vec2.add(Vec2)` ve `Vec2.add(float)` metodlarını karşılayan `Vec2::add_vec2` ve `Vec2::add_scalar` metodlarını içerir. Java oracle doğrudan `add` çağrılarını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1046 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1046/1046** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec2 ve scalar overload'ları, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1548/1548** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1548 senaryoluk Java referansını yeniden doğruladı.

Math referansı `bd99eadfa488c44d624d410a441fe8b99d4485c7236dd276609af1edc3394f3b`, birleşik referans `7a79765a9b449a8ea9c09f049a1ab917958fff879c31b1058ce2f4b8dd3208ab` kimliklerindedir.

Yerel raporlar: `compare-m8v98wom` (math debug), `compare-4i19c_i3` (math release), `compare-p7m9abjy` (birleşik debug), `compare-vvlbkmvf` (birleşik release) ve `verify-_w4izisu` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 girdisi ve add sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 dot, rotate, distance ve diğer yardımcıları ayrı davranış dilimleridir.
