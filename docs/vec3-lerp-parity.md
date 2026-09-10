# P01.b — Vec3 lerp doğrulaması

`src/math.rs`, Java `Vec3.lerp(Vec3, double)` metodunu karşılar. Rust adaptörü her eksende `p0 + alpha * (p1 - p0)` hesabını yapar; Java oracle doğrudan `Vec3.lerp` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1723 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1723/1723** math senaryosunda eşleşti.
- Beş yeni senaryoda temel lerp, negatif ve 1.0 üstü extrapolation, signed zero, subnormal ve özel değer yayılımı karşılaştırıldı.
- Birleşik altı aile corpus'u **2225/2225** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2225 senaryoluk Java referansını yeniden doğruladı.

Math referansı `efa8006dabc939417f5dae848830a2b524d131c4014d44a262a55fc501c1ba27`, birleşik referans `07898354b4bb5d4f13d9fb4841bbe80cf32e162852f5635cbf2013b0de73a161` kimliğindedir.

Yerel raporlar: `compare-vdcnovbh` (math debug), `compare-4w3_lr0t` (math release), `compare-c5b8xhs4` (birleşik debug), `compare-mdqcgmw7` (birleşik release) ve `verify-r95rgxgt` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.lerp` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth` scalar/f32 lerp overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
