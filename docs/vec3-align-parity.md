# P01.b — Vec3 align doğrulaması

`src/math.rs`, Java `Vec3.align(EnumSet<Direction.Axis>)` metodunu karşılar. Rust adaptörü JSON eksen listesini Axis değerlerine çözer ve seçilen bileşenlerde Java floor/cast yolunu kullanır; Java oracle doğrudan `Vec3.align` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1378 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1378/1378** math senaryosunda eşleşti.
- Altı yeni senaryoda boş, kısmi, tüm, tekrarlı ve sıralı eksen kümeleri; negatif kesir, signed zero, subnormal, Infinity ve NaN double-to-int sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1880/1880** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1880 senaryoluk Java referansını yeniden doğruladı.

Math referansı `6d35da84e1eb81c28088bf3f4951651b252daed29eda65c96f035389b897e6e4`, birleşik referans `d1375ef4dcd71b8eb3c95f84ca200ef0ed3303aa7500447f6683cb24a4bd1d81` kimliğindedir. Math karşılaştırma raporları `compare-_ohp5wb8` (debug) ve `compare-bezugwpw` (release); birleşik raporlar `compare-i4o7vofo` (debug) ve `compare-9vhvfokc` (release), Java doğrulaması `verify-kfxghdw3` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.align(EnumSet<Direction.Axis>)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth.floor` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
