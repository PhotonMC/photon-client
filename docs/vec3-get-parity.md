# P01.b — Vec3 get doğrulaması

`src/math.rs`, Java `Vec3.get(Direction.Axis)` metodunu karşılar. Rust adaptörü seçilen eksenin bileşen bitlerini döndürür; Java oracle doğrudan `Vec3.get` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1335 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1335/1335** math senaryosunda eşleşti.
- Beş yeni senaryoda üç eksen, signed zero, subnormal, en büyük sonlu değer, Infinity ve payload taşıyan NaN ham bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1837/1837** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1837 senaryoluk Java referansını yeniden doğruladı.

Math referansı `8956f11f5ab0ba4c81a6aa2c11af8623a498bbb78a550a1b013695afc5029da2`, birleşik referans `f90c38c2c4e4eb83aec92229706d7f6b47bc90a128aa4908ed60897e8279a0d9` kimliğindedir. Math karşılaştırma raporları `compare-0dy1p_jx` (debug) ve `compare-fdzx3gij` (release); birleşik raporlar `compare-gridp_gxu` (debug) ve `compare-u8zwfn4r` (release), Java doğrulaması `verify-yhwxvq8l` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.get(Direction.Axis)` sonucunun ham f64 bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
