# P05.d — SmoothDouble sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, mouse/camera yolunda kullanılan üç alanlı `SmoothDouble` yardımcı durumunu kapsar. MouseHandler'ın hassasiyet dalları, smooth camera seçimi, frame zamanı ve oyuncu rotasyonu sonraki input/kamera dilimlerinde kalır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `util/SmoothDouble.java:3-26` ve `util/Mth.java:558-560` incelendi. Oracle, gerçek Java `SmoothDouble` nesnesi üzerinde sıralı delta/reset çağrılarını çalıştırır.

- Her delta çağrısında target değere `targetDelta` eklenir; mevcut delta target ile remaining farkıdır. Yeni last amount, mevcut last amount ile delta arasında `Mth.lerp(0.5, ...)` ile hesaplanır.
- Delta işareti Java `Math.signum` ile alınır. İşaretli delta büyüklüğü önceki last amount'ın işaretli büyüklüğünü geçiyorsa delta yeni last amount'a çevrilir; sonra last amount güncellenir, remaining'e `delta * time` eklenir ve aynı çarpım döndürülür.
- NaN, Infinity, signed zero, sıfır zaman ve reset değerleri bu işlem sırasının gözlenebilir sonuçlarıdır. `reset` target, remaining ve last amount alanlarını Java `0.0` değerine döndürür.

Rust karşılığı `src/smooth.rs` içindeki `SmoothDouble` tipidir. Java `Math.signum` sıfır/NaN ayrımı ayrı yardımcıyla korunur; MouseHandler entegrasyonu bu veri dilimine dahil değildir.

## JSONL sınırı

İstek `family: "math"`, `operation: "smooth_double"` ve sıralı `delta`/`reset` adımlarını taşır. Delta sonucu f64 raw bitleriyle, reset sonucu işlem adıyla kaydedilir.


## Client-parity doğrulaması

`src/smooth.rs`, Java Edition 26.2 / protokol 776 / Java 25 kaynak davranışına göre SmoothDouble state güncellemesini ve reset yolunu uygular. Java oracle gerçek `SmoothDouble` nesnesini çalıştırır; beklenen sonuç Rust adapter'ından türetilmez.

Math referansı `9980b61aa32b75b8061f04bce2a77c3e588071340884f2137494422f307816dc`; tam corpus referansı `e4d6c2858e052ac5923e2dca299f54838246501a9713bf2cce46e8d74957bd05` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-9h37zp9i` ve `compare-jhnf2bsk`; tam debug/release karşılaştırmaları `compare-ttcbx1il` ve `compare-tiuvlq5d` raporlarında fark üretmedi.

Senaryolar pozitif/negatif delta, farklı zaman çarpanları, sıfır zaman, reset sonrası yeniden başlama, signed zero ve NaN/Infinity akışlarını kapsar. MouseHandler hassasiyet/scope/smooth camera dalları, frame zamanlaması ve oyuncu dönüşü bu doğrulamanın kapsamı dışındadır.

Tekrarlanabilir doğrulama komutları:

```text
python3 tools/behavior/run.py verify
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --adapter target/release/examples/behavior_adapter
python3 tools/behavior/test_runner.py
cargo test --locked --offline --all-targets
cargo clippy --locked --offline --all-targets --all-features -- -D warnings
```
