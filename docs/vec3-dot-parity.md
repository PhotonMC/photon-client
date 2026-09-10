# P01.b — Vec3 dot doğrulaması

`src/math.rs`, Java `Vec3.dot(Vec3)` metodunu karşılayan `Vec3::dot` metodunu içerir. Java oracle doğrudan `dot` çağrısını yapar; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1354 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1354/1354** math senaryosunda eşleşti.
- Beş yeni senaryoda temel çarpım, evaluation order/cancellation, signed zero, double overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1856/1856** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1856 senaryoluk Java referansını yeniden doğruladı.

Math referansı `7bacfa4674e388b5f38f1b0a041e5531afe27e845d4374895c79e4fb56697a24`, birleşik referans `3afb23055305f62e13f7bdef0f8d340c8294187f00de3b26a2c52874e45642a7` kimliğindedir.

Yerel raporlar: `compare-z9ypl4_s` (math debug), `compare-0xhgd0yz` (math release), `compare-ote21q3t` (birleşik debug), `compare-r5nijsku` (birleşik release) ve `verify-2b2qykhl` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 çifti ve dot sonuçlarının f64 bit snapshot'larıyla sınırlıdır. `cross`, `distanceToSqr`, `distanceTo`, closerThan ve diğer Vec3 yardımcıları ayrı dilimlerdir.
