# P01.b — Vec3 align doğrulaması

`src/math.rs`, Java `Vec3.align(EnumSet<Direction.Axis>)` metodunu karşılar. Rust adaptörü JSON eksen listesini Axis değerlerine çözer ve seçilen bileşenlerde Java floor/cast yolunu kullanır; Java oracle doğrudan `Vec3.align` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1040 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1040/1040** math senaryosunda eşleşti.
- Altı yeni senaryoda boş, kısmi, tüm, tekrarlı ve sıralı eksen kümeleri; negatif kesir, signed zero, subnormal, Infinity ve NaN double-to-int sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1542/1542** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1542 senaryoluk Java referansını yeniden doğruladı.

Math referansı `a21163be0f5e47c3d99a4468d2f3b67c6d04d0dcee95ca6b70f49b581fac358c`, birleşik referans `b28d88fa06544e5713186c8c0c44bacbc4c7be8b946c7932a025574a1b3b2126` kimliğindedir. Math karşılaştırma raporları `compare-_rqqaqfb` (debug) ve `compare-v1ltm6cc` (release); birleşik raporlar `compare-1lz8q4ar` (debug) ve `compare-fvdcbn0r` (release), Java doğrulaması `verify-nud8oh4e` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.align(EnumSet<Direction.Axis>)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth.floor` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
