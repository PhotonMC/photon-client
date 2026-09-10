# P01.a — Mth absMax doğrulaması

`src/math.rs`, Java `Mth.absMax(float, float)` ve `absMax(double, double)` yollarını her girdide Java `Math.abs`, ardından Java `Math.max` sırasıyla uygular. Mutlak değer için işaret biti temizlenir; NaN dalları Rust genel `max` davranışına bırakılmaz. `examples/behavior_adapter.rs` sonuçları raw f32/f64 bit string'i olarak taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Her hassasiyet için 14, toplam **28 özgün senaryo** hazırlandı.
- Pozitif/negatif girdiler, eşit mutlak değerler, sıfır ve signed zero, minimum subnormal, maksimum sonlu değer, Infinity ve pozitif/negatif NaN girdileri karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1354/1354** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1856/1856** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `7bacfa4674e388b5f38f1b0a041e5531afe27e845d4374895c79e4fb56697a24`. Birleşik referans kimliği: `3afb23055305f62e13f7bdef0f8d340c8294187f00de3b26a2c52874e45642a7`.

Adapter hashleri: debug `45dccbceba09502c7c6c8e18ca105a628633653b7a27d8cb49b0aba1229bac62`, release `8274db44ba308fe51298d304e3f2dfbd62e8e2a353a0f84b806ee6d3643b26db`.

Yerel raporlar: `compare-z9ypl4_s` ve `compare-0xhgd0yz` (math debug/release), `compare-ote21q3t` ve `compare-r5nijsku` (birleşik debug/release), `verify-33997f0f` (math) ve `verify-2b2qykhl` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca iki `absMax` floating-point overload'unun sonuçlarına ve mevcut corpus regresyonuna aittir. Integer `absMax`, `Mth.abs(int)`, `equal`, positiveModulo, clamp/lerp ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
