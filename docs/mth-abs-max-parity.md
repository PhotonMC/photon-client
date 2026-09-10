# P01.a — Mth absMax doğrulaması

`src/math.rs`, Java `Mth.absMax(float, float)` ve `absMax(double, double)` yollarını her girdide Java `Math.abs`, ardından Java `Math.max` sırasıyla uygular. Mutlak değer için işaret biti temizlenir; NaN dalları Rust genel `max` davranışına bırakılmaz. `examples/behavior_adapter.rs` sonuçları raw f32/f64 bit string'i olarak taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 14, toplam **28 özgün senaryo** hazırlandı.
- Pozitif/negatif girdiler, eşit mutlak değerler, sıfır ve signed zero, minimum subnormal, maksimum sonlu değer, Infinity ve pozitif/negatif NaN girdileri karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1638/1638** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2140/2140** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `18248ddafaffd46fa827971d82704febaf2aa40a94cc0b653078329db1f4cacf`. Birleşik referans kimliği: `1d27ad7d5cb42689c896be0cd7749d1c016b384dacc64f804fb9b17c36a7f38f`.

Adapter hashleri: debug `4a9c082a05009b8ac6c37d12ee8814942a447b8e2e3a445f3ed95942b6a8356e`, release `750a8467b38e832bff19cfcaa43a5a4f94653270c98fd4a4d3f46248f0665f5d`.

Yerel raporlar: `compare-ep8fiu6` ve `compare-z9iwcisf` (math debug/release), `compare-a21c2t4m` ve `compare-uwg2_ewv` (birleşik debug/release), `verify-8mq2_8pm` (math) ve `verify-yscj0_fq` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `absMax` floating-point overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. Integer `absMax`, `Mth.abs(int)`, `equal`, positiveModulo, clamp/lerp ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
