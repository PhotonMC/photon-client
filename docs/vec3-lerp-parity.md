# P01.b — Vec3 lerp doğrulaması

`src/math.rs`, Java `Vec3.lerp(Vec3, double)` metodunu karşılar. Rust adaptörü her eksende `p0 + alpha * (p1 - p0)` hesabını yapar; Java oracle doğrudan `Vec3.lerp` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2023 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2023/2023** math senaryosunda eşleşti.
- Beş yeni senaryoda temel lerp, negatif ve 1.0 üstü extrapolation, signed zero, subnormal ve özel değer yayılımı karşılaştırıldı.
- Birleşik altı aile corpus'u **2525/2525** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2525 senaryoluk Java referansını yeniden doğruladı.

Math referansı `1a217debf64171ea8f99e220ffd6aca1130ecceef3e6015b82f0c12a4e3f9f0d`, birleşik referans `38386c8d844bb2560f83f62571433316aca08b176233a98979cccf45ec22a8d5` kimliğindedir.

Yerel raporlar: `compare-4sx1lm2m` (math debug), `compare-m3ib6sjn` (math release), `compare-czvhou3w` (birleşik debug), `compare-oz19j96e` (birleşik release) ve `verify-nukwd4yc` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.lerp` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth` scalar/f32 lerp overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
