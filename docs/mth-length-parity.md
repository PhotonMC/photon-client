# P01.a — Mth length doğrulaması

`src/math.rs`, Java `Mth.lengthSquared` ve `Mth.length` overload'larını; iki/üç bileşenli double yolları, `length(float, float)` double ara yolunu ve üçlü f32 kare toplamını uygular. `examples/behavior_adapter.rs` sonucu Java oracle ile aynı hassasiyetin raw bit biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- İki/üç bileşenli f64 `lengthSquared` ve `length`, f32 iki bileşenli `length` ve f32 üç bileşenli `lengthSquared` yollarını kapsayan toplam **16 özgün senaryo** hazırlandı.
- 3-4 temel üçgeni, signed zero, Infinity, NaN, double overflow ve f32 overflow girdileriyle birlikte kaynak overload çözümlemesi ve sonuç hassasiyeti karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1578/1578** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2080/2080** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `845bee8faa9bd67215b4c4a330fee9163ef37c01793940d730ec7655ee66c654`. Birleşik referans kimliği: `e1fbdb47e2227938261f4e5bd3238926120301582d50b1af7c84fefac9844a94`.

Adapter hashleri: debug `52aca32c3cc517ab19166f516a9e8edbb6919ddbd9ecf9c522cbbbeedb57a99e`, release `37157540d5781ee33ba26f55c955893baf917f72beac442340eecd884072c5c0`.

Yerel raporlar: `compare-l4cf39d7` ve `compare-jvzkn11s` (math debug/release), `compare-oc9sa8gq` ve `compare-m9ue0q18` (birleşik debug/release), `verify-lwzx0wr7` (math) ve `verify-np1asir2` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca altı `Mth.lengthSquared/length` overload'unun sabit girdili sonuç bitlerine ve mevcut corpus regresyonuna aittir. Vec2/Vec3 uzunluk metotları, `Mth.quantize`, `Mth.outFromOrigin` dışındaki sonraki `Mth` yardımcıları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
