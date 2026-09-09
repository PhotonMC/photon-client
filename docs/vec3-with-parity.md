# P01.b — Vec3 with doğrulaması

`src/math.rs`, Java `Vec3.with(Direction.Axis, double)` metodunu karşılar. Rust adaptörü seçilen bileşeni verilen f64 değere taşır ve diğer bileşen bitlerini korur; Java oracle doğrudan `Vec3.with` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 965 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **965/965** math senaryosunda eşleşti.
- Beş yeni senaryoda X/Y/Z seçimi, değiştirilmemiş bileşenlerin korunması, signed zero, subnormal, en büyük sonlu değer, Infinity ve NaN payload bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1467/1467** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1467 senaryoluk Java referansını yeniden doğruladı.

Math referansı `bbc2c1dc1c594cc962e6acbf6fcdc98815c896f66dcad6a72c2da4be5b0a0f81`, birleşik referans `5cb121b3bc1e12571ad171166a785300fd6e581cf78813e4ddaf2b99d19ab7d1` kimliğindedir. Math karşılaştırma raporları `compare-ctepk6ji` (debug) ve `compare-5wubt_if` (release); birleşik raporlar `compare-lsdsnr89` (debug) ve `compare-b5y_ka7p` (release), Java doğrulaması `verify-o3reu3wd` kimliklerindedir.

## Sınırlar

Kanıt beş özgün sentetik Vec3 girdisinde `Vec3.with(Direction.Axis, double)` sonucunun ham f64 snapshot bitleriyle sınırlıdır. `Direction.Axis.choose` diğer tür overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
