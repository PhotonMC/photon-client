# P01.b — Vec2 equals/hashCode doğrulaması

`src/math.rs`, Java `Vec2.equals(Object)` ve `Vec2.hashCode()` metodlarını karşılayan `Vec2::equals_java` ve `Vec2::hash_code_java` metodlarını içerir. Java oracle doğrudan iki Java metodunu çağırır; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1638 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1638/1638** math senaryosunda eşleşti.
- Altı yeni senaryoda f32 `==`, signed zero/NaN eşitliği, canonical NaN hash'i, Infinity ve 31 tabanlı Java int hash aritmetiği karşılaştırıldı.
- Birleşik altı aile corpus'u **2140/2140** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2140 senaryoluk Java referansını yeniden doğruladı.

Math referansı `18248ddafaffd46fa827971d82704febaf2aa40a94cc0b653078329db1f4cacf`, birleşik referans `1d27ad7d5cb42689c896be0cd7749d1c016b384dacc64f804fb9b17c36a7f38f` kimliklerindedir.

Yerel raporlar: `compare-ep8fiu6` (math debug), `compare-z9iwcisf` (math release), `compare-a21c2t4m` (birleşik debug), `compare-uwg2_ewv` (birleşik release) ve `verify-yscj0_fq` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt altı özgün sentetik Vec2 çifti ve equals/hashCode sonuçlarıyla sınırlıdır. Vec2 negated, add, dot, rotate ve distanceToSqr ayrı davranış dilimleridir.
