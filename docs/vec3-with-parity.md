# P01.b — Vec3 with doğrulaması

`src/math.rs`, Java `Vec3.with(Direction.Axis, double)` metodunu karşılar. Rust adaptörü seçilen bileşeni verilen f64 değere taşır ve diğer bileşen bitlerini korur; Java oracle doğrudan `Vec3.with` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1534 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1534/1534** math senaryosunda eşleşti.
- Beş yeni senaryoda X/Y/Z seçimi, değiştirilmemiş bileşenlerin korunması, signed zero, subnormal, en büyük sonlu değer, Infinity ve NaN payload bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2036/2036** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2036 senaryoluk Java referansını yeniden doğruladı.

Math referansı `99f42388f3562f04669e01818a39202ee8f698f0151f153acf5352e96f0fcc58`, birleşik referans `5607afc8c59b5ee3393ed20e95d9fbca8ce40846df423d5e52589b902d90b90f` kimliğindedir. Math karşılaştırma raporları `compare-t6yvipac` (debug) ve `compare-6_snmnk3` (release); birleşik raporlar `compare-tgx0qoj4` (debug) ve `compare-t1bsi_04` (release), Java doğrulaması `verify-zwp9iban` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.with(Direction.Axis, double)` sonucunun ham f64 snapshot bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
