# P01.b — Vec3 align doğrulaması

`src/math.rs`, Java `Vec3.align(EnumSet<Direction.Axis>)` metodunu karşılar. Rust adaptörü JSON eksen listesini Axis değerlerine çözer ve seçilen bileşenlerde Java floor/cast yolunu kullanır; Java oracle doğrudan `Vec3.align` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1703 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1703/1703** math senaryosunda eşleşti.
- Altı yeni senaryoda boş, kısmi, tüm, tekrarlı ve sıralı eksen kümeleri; negatif kesir, signed zero, subnormal, Infinity ve NaN double-to-int sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2205/2205** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2205 senaryoluk Java referansını yeniden doğruladı.

Math referansı `b93cda4c5257c03011378a24fe916e8b8cd7e11d2fe75e1a1de108e4b4ebd630`, birleşik referans `413b981c2a619373545c6c4c74d3e7fec6b1a2e1dc34d5a49ee2f4eed5231162` kimliğindedir. Math karşılaştırma raporları `compare-a8qhmnrl` (debug) ve `compare-usm27y8z` (release); birleşik raporlar `compare-8vwtvwwl` (debug) ve `compare-uey51w68` (release), Java doğrulaması `verify-l3irw60r` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.align(EnumSet<Direction.Axis>)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `Mth.floor` bağımsız overload'ları ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
