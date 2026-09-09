# P01.b — Vec2 dot doğrulaması

`src/math.rs`, Java `Vec2.dot(Vec2)` metodunu karşılayan `Vec2::dot` metodunu içerir. Java oracle doğrudan `dot` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1007 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1007/1007** math senaryosunda eşleşti.
- Beş yeni senaryoda temel çarpım, f32 yuvarlama, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1509/1509** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1509 senaryoluk Java referansını yeniden doğruladı.

Math referansı `7b8b9ea0f5717fc78d7092f1d3563134bfdae91e1c70cf411bb3269f1ad58d4d`, birleşik referans `f71b676b2ae17505734ec3d87f208055714fa5dd6b9cc9417d155abf92e2ac0a` kimliklerindedir.

Yerel raporlar: `compare-upfzoh9c` (math debug), `compare-czpd1pgo` (math release), `compare-_3h0xf61` (birleşik debug), `compare-a2h2wcm2` (birleşik release) ve `verify-19j4rwux` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 çifti ve dot sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 add, scale, normalize, distance ve diğer yardımcıları ayrı davranış dilimleridir.
