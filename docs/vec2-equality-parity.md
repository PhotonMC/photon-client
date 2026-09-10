# P01.b — Vec2 equals/hashCode doğrulaması

`src/math.rs`, Java `Vec2.equals(Object)` ve `Vec2.hashCode()` metodlarını karşılayan `Vec2::equals_java` ve `Vec2::hash_code_java` metodlarını içerir. Java oracle doğrudan iki Java metodunu çağırır; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1478 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1478/1478** math senaryosunda eşleşti.
- Altı yeni senaryoda f32 `==`, signed zero/NaN eşitliği, canonical NaN hash'i, Infinity ve 31 tabanlı Java int hash aritmetiği karşılaştırıldı.
- Birleşik altı aile corpus'u **1980/1980** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1980 senaryoluk Java referansını yeniden doğruladı.

Math referansı `abe3d8cdf014db1ee9b2fa953ff1b7ba7bc40e31c9d35672c0cd8adb20896f50`, birleşik referans `e068e38cc1d0ab64ac2844f20d474f313e2325b42cc620af39051ca8aa6748cd` kimliklerindedir.

Yerel raporlar: `compare-xrmyz06w` (math debug), `compare-zrxwzjp_` (math release), `compare-dxdmken0` (birleşik debug), `compare-cvif6gdr` (birleşik release) ve `verify-jlkp_8wf` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt altı özgün sentetik Vec2 çifti ve equals/hashCode sonuçlarıyla sınırlıdır. Vec2 negated, add, dot, rotate ve distanceToSqr ayrı davranış dilimleridir.
