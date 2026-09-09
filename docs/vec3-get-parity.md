# P01.b — Vec3 get doğrulaması

`src/math.rs`, Java `Vec3.get(Direction.Axis)` metodunu karşılar. Rust adaptörü seçilen eksenin bileşen bitlerini döndürür; Java oracle doğrudan `Vec3.get` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1149 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1149/1149** math senaryosunda eşleşti.
- Beş yeni senaryoda üç eksen, signed zero, subnormal, en büyük sonlu değer, Infinity ve payload taşıyan NaN ham bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1651/1651** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1651 senaryoluk Java referansını yeniden doğruladı.

Math referansı `2f973f986814298b094b982753eef5a170865c5228762b233a6060766b5ed32e`, birleşik referans `f8cfd3cf28e07e5859a339a4844b50e474cb8351ea003bb36ba96ccaf38abec1` kimliğindedir. Math karşılaştırma raporları `compare-hz521aor` (debug) ve `compare-uded0zec` (release); birleşik raporlar `compare-nc1496mg` (debug) ve `compare-ljx3yrp0` (release), Java doğrulaması `verify-k950tc5l` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.get(Direction.Axis)` sonucunun ham f64 bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
