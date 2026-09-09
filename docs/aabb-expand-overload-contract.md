# P01.d — AABB.expandTowards(Vec3) sözleşmesi

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-expand-overload-parity.md)). Kapsam, `AABB.expandTowards(Vec3)` overload'ının bileşenleri scalar `expandTowards(double, double, double)` yoluna aktarmasını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:157-163` overload yönlendirmesi ve `AABB.java:161-189` scalar genişletme yolu incelendi. `MathOracle`, Vec3 overload'ını doğrudan çağırır; beklenen kutu yeniden hesaplanmaz.

## Davranış sözleşmesi

- Vec3 overload'ı `delta.x`, `delta.y` ve `delta.z` bileşenlerini scalar overload'a aynı sırayla verir.
- Pozitif delta ilgili eksenin max endpoint'ini, negatif delta min endpoint'ini değiştirir; sıfır endpoint'i korur.
- Sonuç constructor normalizasyonu ve Java double bit davranışıyla taşınır; signed zero girdisi korunur.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `vec3` | Pozitif X, negatif Y ve sıfır Z delta'sı | Genişletilmiş AABB snapshot'ı |
| `signed-zero` | Signed-zero endpoint'li kutu ve zero Vec3 | Endpoint bitleri ve değişmeyen kutu |
| `chain` | İki ardışık Vec3 genişletme adımı | Her adım sonrası AABB snapshot'ı |

Girdiler `family: "math"`, `operation: "aabb"` ve `steps` içindeki `op: "expand_vec3"` alanlarını kullanır. Vec3 bileşenleri f64 bit string'leridir; `events` başlangıç ve her adım sonrası kutuyu taşır.

## Java yakalama

Math ve birleşik altı aile referansı `b5e20f42ea20b904f73ce2c8cd50b3c271f9e5405ad39815c2cbd61da6b7fb92` kimliğiyle iki JVM çalıştırmasında doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
