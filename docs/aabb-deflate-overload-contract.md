# P01.d — AABB deflate üç eksenli overload sözleşmesi

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-deflate-overload-parity.md)). Kapsam, Java `AABB.deflate(double, double, double)` overload'unun eksen başına miktarları `inflate` yoluna ters işaretle aktarmasını içerir. Scalar `deflate(double)` ve diğer AABB işlemleri ayrı senaryolarla izlenir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:286-292` incelendi. `MathOracle`, üç f64 miktarı doğrudan Java `AABB.deflate` overload'una verir; beklenen uçlar Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Üç eksenli overload her miktarın işaretini Java unary negation ile çevirip `inflate` üç eksenli yoluna aktarır.
- Pozitif miktar alt ve üst endpoint'leri içeri taşır; negatif miktar kutuyu o eksende genişletir.
- Miktarlar eksen başına bağımsızdır. Sonuç Java `AABB` kurucusunun endpoint normalizasyonundan geçer.
- Signed zero, NaN ve sonsuzluk değerleri Java double negation ve aritmetik davranışıyla korunur; Rust çağrı yolu debug ve release'te aynı bitleri üretir.
- Ardışık adımlar önceki AABB sonucunu sonraki `deflate_xyz` veya scalar `deflate` çağrısına verir.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vector-mixed` | Karışık işaretli üç eksen miktarı | Eksen başına endpoint sonucu |
| `vector-reversed` | Boyutu aşan pozitif miktarlar | Constructor normalizasyonu |
| `signed-zero` | Signed zero miktarları | Endpoint bitleri |
| `chain` | Üç eksenli ve scalar ardışık adımları | Sıralı AABB snapshot'ları |
| `special-values` | NaN ve sonsuzluk | Java özel değer bitleri |

Girdiler `family: "math"`, `operation: "aabb"` ve `steps` içinde `op: "deflate_xyz"` kullanır. Miktarlar f64 bit string'leridir; her adım sonrası kutu endpoint'leri `events` içinde taşınır.

## Rust karşılığı

`src/aabb.rs` içindeki `Aabb::deflate_xyz`, Java'nın ters işaretli `inflate` yönlendirmesini uygular. Java unary negation'ın NaN işaretini release optimizasyonlarından bağımsız korumak için bit düzeyinde yardımcı kullanılır. `examples/behavior_adapter.rs` bu metodu JSONL `deflate_xyz` adımına bağlar.
