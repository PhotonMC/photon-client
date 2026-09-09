# P01.b — Vec3 get doğrulaması

`src/math.rs`, Java `Vec3.get(Direction.Axis)` metodunu karşılar. Rust adaptörü seçilen eksenin bileşen bitlerini döndürür; Java oracle doğrudan `Vec3.get` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 730 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **730/730** math senaryosunda eşleşti.
- Beş yeni senaryoda üç eksen, signed zero, subnormal, en büyük sonlu değer, Infinity ve payload taşıyan NaN ham bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1184/1184** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1184 senaryoluk Java referansını yeniden doğruladı.

Math referansı `bf7aa5989fd29e9e312e3f8dfe28a7813e91710a88abbd6cb7631842b93b63e8`, birleşik referans `b90f668d7f2493a73a435971a72ec7d819d9f40d72b9c7ae79c7a1a6e9dd76f7` kimliğindedir. Math karşılaştırma raporları `compare-cjx12cp7` (debug) ve `compare-br16mrsl` (release); birleşik raporlar `compare-koctdm6s` (debug) ve `compare-8rx1p5q8` (release), Java doğrulaması `verify-ebvdfs33` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.get(Direction.Axis)` sonucunun ham f64 bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
