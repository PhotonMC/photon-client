# P01.b — Vec2 add overload'ları doğrulaması

`src/math.rs`, Java `Vec2.add(Vec2)` ve `Vec2.add(float)` metodlarını karşılayan `Vec2::add_vec2` ve `Vec2::add_scalar` metodlarını içerir. Java oracle doğrudan `add` çağrılarını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1149 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1149/1149** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec2 ve scalar overload'ları, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1651/1651** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1651 senaryoluk Java referansını yeniden doğruladı.

Math referansı `2f973f986814298b094b982753eef5a170865c5228762b233a6060766b5ed32e`, birleşik referans `f8cfd3cf28e07e5859a339a4844b50e474cb8351ea003bb36ba96ccaf38abec1` kimliklerindedir.

Yerel raporlar: `compare-hz521aor` (math debug), `compare-uded0zec` (math release), `compare-nc1496mg` (birleşik debug), `compare-ljx3yrp0` (birleşik release) ve `verify-k950tc5l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 girdisi ve add sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 dot, rotate, distance ve diğer yardımcıları ayrı davranış dilimleridir.
