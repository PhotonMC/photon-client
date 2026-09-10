# P01.a — Mth length doğrulaması

`src/math.rs`, Java `Mth.lengthSquared` ve `Mth.length` overload'larını; iki/üç bileşenli double yolları, `length(float, float)` double ara yolunu ve üçlü f32 kare toplamını uygular. `examples/behavior_adapter.rs` sonucu Java oracle ile aynı hassasiyetin raw bit biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- İki/üç bileşenli f64 `lengthSquared` ve `length`, f32 iki bileşenli `length` ve f32 üç bileşenli `lengthSquared` yollarını kapsayan toplam **16 özgün senaryo** hazırlandı.
- 3-4 temel üçgeni, signed zero, Infinity, NaN, double overflow ve f32 overflow girdileriyle birlikte kaynak overload çözümlemesi ve sonuç hassasiyeti karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1668/1668** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2170/2170** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `a025973c1ef8351cb5437b997d6b70b36792edd52713c0b5f087210263fec060`. Birleşik referans kimliği: `0ab94c7aca8ab0f093d23176b8a698767746ba84238949d19b77f4b5732d94d7`.

Adapter hashleri: debug `0feff16b39b41d5108009538a31c8ca27bae1eb27830e05f355e249e825a3c59`, release `c26e0d6d9adf0985a9f18bdf551d783dddaf31550ac373bdc7233cdebc70707f`.

Yerel raporlar: `compare-di4orhey` ve `compare-9os9l34y` (math debug/release), `compare-spny1_zm` ve `compare-bu52w5ts` (birleşik debug/release), `verify-dh3f392l` (math) ve `verify-j7u8aby_` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca altı `Mth.lengthSquared/length` overload'unun sabit girdili sonuç bitlerine ve mevcut corpus regresyonuna aittir. Vec2/Vec3 uzunluk metotları, `Mth.quantize`, `Mth.outFromOrigin` dışındaki sonraki `Mth` yardımcıları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
