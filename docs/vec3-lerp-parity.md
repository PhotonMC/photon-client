# P01.b — Vec3 lerp doğrulaması

`src/math.rs`, Java `Vec3.lerp(Vec3, double)` metodunu karşılar. Rust adaptörü her eksende `p0 + alpha * (p1 - p0)` hesabını yapar; Java oracle doğrudan `Vec3.lerp` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2100 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2100/2100** math senaryosunda eşleşti.
- Beş yeni senaryoda temel lerp, negatif ve 1.0 üstü extrapolation, signed zero, subnormal ve özel değer yayılımı karşılaştırıldı.
- Birleşik altı aile corpus'u **2602/2602** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2602 senaryoluk Java referansını yeniden doğruladı.

Math referansı `f84b8d0a8b920d9e5455aefeebd2a9d768b7c9af5ace5dd1c90ff816a9e20abb`, birleşik referans `ec46e2f5bb40c4fdc045039ffcf03e500f593e7a5659dcd76ba9519601df03f2` kimliğindedir.

Yerel raporlar: `compare-f8rnhap6` (math debug), `compare-9ykm10o3` (math release), `compare-qo27kk4p` (birleşik debug), `compare-066pmptb` (birleşik release) ve `verify-icag4r7w` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.lerp` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth` scalar/f32 lerp overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
