# P05.d — SmoothDouble sözleşmesi

Bu dilim, mouse/camera yolunda kullanılan üç alanlı `SmoothDouble` yardımcı durumunu kapsar. MouseHandler'ın hassasiyet dalları, smooth camera seçimi, frame zamanı ve oyuncu rotasyonu sonraki input/kamera dilimlerinde kalır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `util/SmoothDouble.java:3-26` ve `util/Mth.java:558-560` incelendi. Oracle, gerçek Java `SmoothDouble` nesnesi üzerinde sıralı delta/reset çağrılarını çalıştırır.

- Her delta çağrısında target değere `targetDelta` eklenir; mevcut delta target ile remaining farkıdır. Yeni last amount, mevcut last amount ile delta arasında `Mth.lerp(0.5, ...)` ile hesaplanır.
- Delta işareti Java `Math.signum` ile alınır. İşaretli delta büyüklüğü önceki last amount'ın işaretli büyüklüğünü geçiyorsa delta yeni last amount'a çevrilir; sonra last amount güncellenir, remaining'e `delta * time` eklenir ve aynı çarpım döndürülür.
- NaN, Infinity, signed zero, sıfır zaman ve reset değerleri bu işlem sırasının gözlenebilir sonuçlarıdır. `reset` target, remaining ve last amount alanlarını Java `0.0` değerine döndürür.

Rust karşılığı `src/smooth.rs` içindeki `SmoothDouble` tipidir. Java `Math.signum` sıfır/NaN ayrımı ayrı yardımcıyla korunur; MouseHandler entegrasyonu bu veri dilimine dahil değildir.

## JSONL sınırı

İstek `family: "math"`, `operation: "smooth_double"` ve sıralı `delta`/`reset` adımlarını taşır. Delta sonucu f64 raw bitleriyle, reset sonucu işlem adıyla kaydedilir.
