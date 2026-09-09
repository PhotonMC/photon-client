# P01.b — Vec3 lerp doğrulaması

`src/math.rs`, Java `Vec3.lerp(Vec3, double)` metodunu karşılar. Rust adaptörü her eksende `p0 + alpha * (p1 - p0)` hesabını yapar; Java oracle doğrudan `Vec3.lerp` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1007 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1007/1007** math senaryosunda eşleşti.
- Beş yeni senaryoda temel lerp, negatif ve 1.0 üstü extrapolation, signed zero, subnormal ve özel değer yayılımı karşılaştırıldı.
- Birleşik altı aile corpus'u **1509/1509** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1509 senaryoluk Java referansını yeniden doğruladı.

Math referansı `7b8b9ea0f5717fc78d7092f1d3563134bfdae91e1c70cf411bb3269f1ad58d4d`, birleşik referans `f71b676b2ae17505734ec3d87f208055714fa5dd6b9cc9417d155abf92e2ac0a` kimliğindedir.

Yerel raporlar: `compare-upfzoh9c` (math debug), `compare-czpd1pgo` (math release), `compare-_3h0xf61` (birleşik debug), `compare-a2h2wcm2` (birleşik release) ve `verify-19j4rwux` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.lerp` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth` scalar/f32 lerp overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
