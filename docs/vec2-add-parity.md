# P01.b — Vec2 add overload'ları doğrulaması

`src/math.rs`, Java `Vec2.add(Vec2)` ve `Vec2.add(float)` metodlarını karşılayan `Vec2::add_vec2` ve `Vec2::add_scalar` metodlarını içerir. Java oracle doğrudan `add` çağrılarını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1903 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1903/1903** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec2 ve scalar overload'ları, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2405/2405** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2405 senaryoluk Java referansını yeniden doğruladı.

Math referansı `0cc821a10c9af88c8563c153e55d57b691a8c69e33d90a8bd3022a5b3410bfd0`, birleşik referans `2daaa6bb28eca57adfb18563e32f011d764209b98ec6de5934a92a72e80abdfa` kimliklerindedir.

Yerel raporlar: `compare-xiw125c3` (math debug), `compare-v5h63z7q` (math release), `compare-chdvlz2_` (birleşik debug), `compare-oywxz0ky` (birleşik release) ve `verify-da0drjib` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 girdisi ve add sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 dot, rotate, distance ve diğer yardımcıları ayrı davranış dilimleridir.
