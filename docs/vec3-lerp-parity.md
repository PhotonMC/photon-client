# P01.b — Vec3 lerp doğrulaması

`src/math.rs`, Java `Vec3.lerp(Vec3, double)` metodunu karşılar. Rust adaptörü her eksende `p0 + alpha * (p1 - p0)` hesabını yapar; Java oracle doğrudan `Vec3.lerp` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1987 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1987/1987** math senaryosunda eşleşti.
- Beş yeni senaryoda temel lerp, negatif ve 1.0 üstü extrapolation, signed zero, subnormal ve özel değer yayılımı karşılaştırıldı.
- Birleşik altı aile corpus'u **2489/2489** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2489 senaryoluk Java referansını yeniden doğruladı.

Math referansı `665bdda6827e71f559f4e9a959ed126f3d3e055594f3b677359c51d0d3a74e16`, birleşik referans `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc` kimliğindedir.

Yerel raporlar: `compare-zt_h_egg` (math debug), `compare-7afcvx21` (math release), `compare-251aed0_` (birleşik debug), `compare-jqj2kta_` (birleşik release) ve `verify-csdh4b83` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.lerp` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth` scalar/f32 lerp overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
