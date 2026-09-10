# P01.b — Vec2 distanceToSqr doğrulaması

`src/math.rs`, Java `Vec2.distanceToSqr(Vec2)` metodunu karşılayan `Vec2::distance_to_sqr` metodunu içerir. Java oracle doğrudan `distanceToSqr` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1999 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1999/1999** math senaryosunda eşleşti.
- Beş yeni senaryoda f32 çıkarma, kare alma, kaynak sıralı toplama, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2501/2501** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2501 senaryoluk Java referansını yeniden doğruladı.

Math referansı `d0cf0bb203c8ea48ad8144fae6a58022846d1a5cfb95b398018620bae7b0bc3d`, birleşik referans `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040` kimliklerindedir.

Yerel raporlar: `compare-druvbg3h` (math debug), `compare-k2ed8y70` (math release), `compare-h1kbcaxz` (birleşik debug), `compare-js7_mas4` (birleşik release) ve `verify-d2ptfzig` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 çifti ve distanceToSqr sonuçlarının f32 bit değerleriyle sınırlıdır. Vec2 add, dot, rotate ve diğer yardımcıları ayrı davranış dilimleridir.
