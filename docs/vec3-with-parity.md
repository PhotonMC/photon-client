# P01.b — Vec3 with doğrulaması

`src/math.rs`, Java `Vec3.with(Direction.Axis, double)` metodunu karşılar. Rust adaptörü seçilen bileşeni verilen f64 değere taşır ve diğer bileşen bitlerini korur; Java oracle doğrudan `Vec3.with` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1301 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1301/1301** math senaryosunda eşleşti.
- Beş yeni senaryoda X/Y/Z seçimi, değiştirilmemiş bileşenlerin korunması, signed zero, subnormal, en büyük sonlu değer, Infinity ve NaN payload bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1803/1803** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1803 senaryoluk Java referansını yeniden doğruladı.

Math referansı `a7681fdbc73c8faa861a3c0f74be7f2286257b8c494b6f1d3c85598ad65f6973`, birleşik referans `e26e5fb4cdc24beb08ca189d4d55904fcff008d93a9a99a606f787f60554f935` kimliğindedir. Math karşılaştırma raporları `compare-zq98bs3k` (debug) ve `compare-zgg863t0` (release); birleşik raporlar `compare-yz31bmcf` (debug) ve `compare-90dzj4qp` (release), Java doğrulaması `verify-l29xakrv` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.with(Direction.Axis, double)` sonucunun ham f64 snapshot bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
