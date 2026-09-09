# P01.b — Vec3 get doğrulaması

`src/math.rs`, Java `Vec3.get(Direction.Axis)` metodunu karşılar. Rust adaptörü seçilen eksenin bileşen bitlerini döndürür; Java oracle doğrudan `Vec3.get` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 735 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **735/735** math senaryosunda eşleşti.
- Beş yeni senaryoda üç eksen, signed zero, subnormal, en büyük sonlu değer, Infinity ve payload taşıyan NaN ham bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1189/1189** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1189 senaryoluk Java referansını yeniden doğruladı.

Math referansı `94a6025ce62400506c0736719b21030eeb6b7d69b6ad0d5d1f2e9c8c3af90491`, birleşik referans `e1537e5e73915e7c752cf9b583382949d05feaca1a439d67f330b3513cb7483e` kimliğindedir. Math karşılaştırma raporları `compare-iadkiiub` (debug) ve `compare-f0hxm0vh` (release); birleşik raporlar `compare-wlecne0c` (debug) ve `compare-uvna2fit` (release), Java doğrulaması `verify-ihxsw2pd` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.get(Direction.Axis)` sonucunun ham f64 bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
