# P01.b — Vec2 equals/hashCode doğrulaması

`src/math.rs`, Java `Vec2.equals(Object)` ve `Vec2.hashCode()` metodlarını karşılayan `Vec2::equals_java` ve `Vec2::hash_code_java` metodlarını içerir. Java oracle doğrudan iki Java metodunu çağırır; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 953 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **953/953** math senaryosunda eşleşti.
- Altı yeni senaryoda f32 `==`, signed zero/NaN eşitliği, canonical NaN hash'i, Infinity ve 31 tabanlı Java int hash aritmetiği karşılaştırıldı.
- Birleşik altı aile corpus'u **1455/1455** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1455 senaryoluk Java referansını yeniden doğruladı.

Math referansı `c05b4f6d5d95b034d9034387072c87ac36f1b31427b1fbbbbebe885dec1bfc3b`, birleşik referans `e267ec135794f878c9e54d22faa3451ffe125c25ad17b7ad67e223c1d1fb3b55` kimliklerindedir.

Yerel raporlar: `compare-j1r42nvx` (math debug), `compare-yls3sbu1` (math release), `compare-ttkrbj5a` (birleşik debug), `compare-dz8xzceq` (birleşik release) ve `verify-zgd9uosd` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt altı özgün sentetik Vec2 çifti ve equals/hashCode sonuçlarıyla sınırlıdır. Vec2 negated, add, dot, rotate ve distanceToSqr ayrı davranış dilimleridir.
