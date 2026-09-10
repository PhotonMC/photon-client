# P01.b — Vec3 get doğrulaması

`src/math.rs`, Java `Vec3.get(Direction.Axis)` metodunu karşılar. Rust adaptörü seçilen eksenin bileşen bitlerini döndürür; Java oracle doğrudan `Vec3.get` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1608 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1608/1608** math senaryosunda eşleşti.
- Beş yeni senaryoda üç eksen, signed zero, subnormal, en büyük sonlu değer, Infinity ve payload taşıyan NaN ham bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2110/2110** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2110 senaryoluk Java referansını yeniden doğruladı.

Math referansı `10c179a43201ead1d005dbd3e8f1deb7d1b4d5928bd228214792c84e07ef2b41`, birleşik referans `bb3698ca7b34e45053dd8865ed3b754080b1fb665c802937b38ccb0216ecede3` kimliğindedir. Math karşılaştırma raporları `compare-pmcommj0` (debug) ve `compare-4g5l34ea` (release); birleşik raporlar `compare-mki3lw11` (debug) ve `compare-r66knife` (release), Java doğrulaması `verify-35w0iz71` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.get(Direction.Axis)` sonucunun ham f64 bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
