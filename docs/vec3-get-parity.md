# P01.b — Vec3 get doğrulaması

`src/math.rs`, Java `Vec3.get(Direction.Axis)` metodunu karşılar. Rust adaptörü seçilen eksenin bileşen bitlerini döndürür; Java oracle doğrudan `Vec3.get` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 759 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **759/759** math senaryosunda eşleşti.
- Beş yeni senaryoda üç eksen, signed zero, subnormal, en büyük sonlu değer, Infinity ve payload taşıyan NaN ham bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1213/1213** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1213 senaryoluk Java referansını yeniden doğruladı.

Math referansı `c620da29c481415908c488545e4f8719444d3ed30d1ae6f9fceaef57a4afab3a`, birleşik referans `dcbe4a62af107a8ef7ed2c1dd52518e6b9e2f063bfc9f0c0ba5565b0ba4617f8` kimliğindedir. Math karşılaştırma raporları `compare-iodxkzgj` (debug) ve `compare-_orh4azt` (release); birleşik raporlar `compare-xeqdw852` (debug) ve `compare-bk_l5btc` (release), Java doğrulaması `verify-ho8canxc` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.get(Direction.Axis)` sonucunun ham f64 bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
