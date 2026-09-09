# P01.b — Vec3 multiply overload doğrulaması

`src/math.rs`, Java `Vec3.multiply(Vec3)` ve `Vec3.multiply(double, double, double)` overload'larının bileşen yönlendirmesini ve x/y/z çarpımlarını karşılar. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1067 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1067/1067** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, overload zinciri, signed zero, taşma ve NaN/Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1569/1569** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1569 senaryoluk Java referansını yeniden doğruladı.

Math referansı `516108a2035e89e9716bb1dcd67352eaa358d73159f8e8391e2d51372d77d713`, birleşik referans `9f76d4ed1ee97b0d89ef969b0fdf139f7e658f9316895af88b889f790f13b8d1` kimliğindedir.

Yerel raporlar: `compare-d3vh9fy0` (math debug), `compare-cdl_uz9p` (math release), `compare-b4u1_5xx` (birleşik debug), `compare-lz2txc82` (birleşik release) ve `verify-i4bptixz` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `scale`, `reverse`, `horizontal`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
