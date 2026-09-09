# P01.b — Vec3 lerp doğrulaması

`src/math.rs`, Java `Vec3.lerp(Vec3, double)` metodunu karşılar. Rust adaptörü her eksende `p0 + alpha * (p1 - p0)` hesabını yapar; Java oracle doğrudan `Vec3.lerp` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1046 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1046/1046** math senaryosunda eşleşti.
- Beş yeni senaryoda temel lerp, negatif ve 1.0 üstü extrapolation, signed zero, subnormal ve özel değer yayılımı karşılaştırıldı.
- Birleşik altı aile corpus'u **1548/1548** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1548 senaryoluk Java referansını yeniden doğruladı.

Math referansı `bd99eadfa488c44d624d410a441fe8b99d4485c7236dd276609af1edc3394f3b`, birleşik referans `7a79765a9b449a8ea9c09f049a1ab917958fff879c31b1058ce2f4b8dd3208ab` kimliğindedir.

Yerel raporlar: `compare-m8v98wom` (math debug), `compare-4i19c_i3` (math release), `compare-p7m9abjy` (birleşik debug), `compare-vvlbkmvf` (birleşik release) ve `verify-_w4izisu` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.lerp` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth` scalar/f32 lerp overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
