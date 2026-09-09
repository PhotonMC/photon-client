# P01.b — Vec3 align doğrulaması

`src/math.rs`, Java `Vec3.align(EnumSet<Direction.Axis>)` metodunu karşılar. Rust adaptörü JSON eksen listesini Axis değerlerine çözer ve seçilen bileşenlerde Java floor/cast yolunu kullanır; Java oracle doğrudan `Vec3.align` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 771 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **771/771** math senaryosunda eşleşti.
- Altı yeni senaryoda boş, kısmi, tüm, tekrarlı ve sıralı eksen kümeleri; negatif kesir, signed zero, subnormal, Infinity ve NaN double-to-int sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1225/1225** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1225 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ffefa3418c35a5dbf05a16b8c6bb664f38665423ada7b039029bea24f0e3f8ac`, birleşik referans `c8d3f97682c7983d7226696163a3943fe4ef6723170037ad5205d10ed0494654` kimliğindedir. Math karşılaştırma raporları `compare-nyuxtnwe` (debug) ve `compare-c83i9uhh` (release); birleşik raporlar `compare-2v85255z` (debug) ve `compare-4_se5ekn` (release), Java doğrulaması `verify-q8mzlbkq` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.align(EnumSet<Direction.Axis>)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth.floor` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
