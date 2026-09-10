# P01.b — Vec2 equals/hashCode doğrulaması

`src/math.rs`, Java `Vec2.equals(Object)` ve `Vec2.hashCode()` metodlarını karşılayan `Vec2::equals_java` ve `Vec2::hash_code_java` metodlarını içerir. Java oracle doğrudan iki Java metodunu çağırır; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2216 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2216/2216** math senaryosunda eşleşti.
- Altı yeni senaryoda f32 `==`, signed zero/NaN eşitliği, canonical NaN hash'i, Infinity ve 31 tabanlı Java int hash aritmetiği karşılaştırıldı.
- Birleşik altı aile corpus'u **2718/2718** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2718 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt altı özgün sentetik Vec2 çifti ve equals/hashCode sonuçlarıyla sınırlıdır. Vec2 negated, add, dot, rotate ve distanceToSqr ayrı davranış dilimleridir.
