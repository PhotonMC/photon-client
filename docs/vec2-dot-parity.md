# P01.b — Vec2 dot doğrulaması

`src/math.rs`, Java `Vec2.dot(Vec2)` metodunu karşılayan `Vec2::dot` metodunu içerir. Java oracle doğrudan `dot` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1067 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1067/1067** math senaryosunda eşleşti.
- Beş yeni senaryoda temel çarpım, f32 yuvarlama, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1569/1569** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1569 senaryoluk Java referansını yeniden doğruladı.

Math referansı `516108a2035e89e9716bb1dcd67352eaa358d73159f8e8391e2d51372d77d713`, birleşik referans `9f76d4ed1ee97b0d89ef969b0fdf139f7e658f9316895af88b889f790f13b8d1` kimliklerindedir.

Yerel raporlar: `compare-d3vh9fy0` (math debug), `compare-cdl_uz9p` (math release), `compare-b4u1_5xx` (birleşik debug), `compare-lz2txc82` (birleşik release) ve `verify-i4bptixz` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 çifti ve dot sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 add, scale, normalize, distance ve diğer yardımcıları ayrı davranış dilimleridir.
