# P01.b — Vec2 equals/hashCode doğrulaması

`src/math.rs`, Java `Vec2.equals(Object)` ve `Vec2.hashCode()` metodlarını karşılayan `Vec2::equals_java` ve `Vec2::hash_code_java` metodlarını içerir. Java oracle doğrudan iki Java metodunu çağırır; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1593 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1593/1593** math senaryosunda eşleşti.
- Altı yeni senaryoda f32 `==`, signed zero/NaN eşitliği, canonical NaN hash'i, Infinity ve 31 tabanlı Java int hash aritmetiği karşılaştırıldı.
- Birleşik altı aile corpus'u **2095/2095** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2095 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ec7f2431c2fc0a873c374068f226b4309821be58380c7e5eadccdc15fe690723`, birleşik referans `17d40db8607a5df22fc08a42ccd0065b6ffe818cef98507474d8663297c0ac04` kimliklerindedir.

Yerel raporlar: `compare-gqgspijr` (math debug), `compare-z8zpkwnx` (math release), `compare-94x3ynna` (birleşik debug), `compare-2cii6gte` (birleşik release) ve `verify-lu6dvbz4` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt altı özgün sentetik Vec2 çifti ve equals/hashCode sonuçlarıyla sınırlıdır. Vec2 negated, add, dot, rotate ve distanceToSqr ayrı davranış dilimleridir.
