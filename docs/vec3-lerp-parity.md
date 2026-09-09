# P01.b — Vec3 lerp doğrulaması

`src/math.rs`, Java `Vec3.lerp(Vec3, double)` metodunu karşılar. Rust adaptörü her eksende `p0 + alpha * (p1 - p0)` hesabını yapar; Java oracle doğrudan `Vec3.lerp` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1019 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1019/1019** math senaryosunda eşleşti.
- Beş yeni senaryoda temel lerp, negatif ve 1.0 üstü extrapolation, signed zero, subnormal ve özel değer yayılımı karşılaştırıldı.
- Birleşik altı aile corpus'u **1521/1521** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1521 senaryoluk Java referansını yeniden doğruladı.

Math referansı `5df2c7aa290a2d3deb2e318553a41a27dbe6b08b22e1d31b68362eedc36e2108`, birleşik referans `53cb6b5a4397567e26419e7466bcd8577c97308e626648d85f2a40bb55fb2406` kimliğindedir.

Yerel raporlar: `compare-_lwapdio` (math debug), `compare-ed7p25g6` (math release), `compare-uh_quimp` (birleşik debug), `compare-233_yr3a` (birleşik release) ve `verify-wqacm20n` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.lerp` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth` scalar/f32 lerp overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
