# P05.d — ScrollWheelHandler sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, mouse scroll offset'lerinin tam wheel adımlarına çevrilmesini ve hotbar seçim helper'ını kapsar. GLFW callback sırası, pencere odağı, GUI/player/spectator yönlendirmesi ve MouseHandler'ın frame içi hareket işleme akışı sonraki input/GUI/world dilimlerinde kalır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/ScrollWheelHandler.java:5-45` incelendi. Oracle, gerçek Java `ScrollWheelHandler` nesnesi ve static selection helper'ı üzerinde kontrollü f64 girdileri çalıştırır.

- Birikmiş eksen sıfır değilken yeni offset'in Java `Math.signum` sonucu farklıysa o eksenin birikimi sıfırlanır. Sonra offset eklenir; double değer Java int dönüşümüyle sıfıra doğru kesilir.
- Her iki tam wheel çıktısı da sıfırsa artık double değerler korunur. En az bir eksen tam adıma ulaştığında tam adımlar döndürülür ve ilgili artık değerlerden çıkarılır; diğer eksenin kesirli artığı aynı çağrıda korunur.
- Selection helper, wheel işaretini Java `Math.signum` ile int adıma çevirip mevcut seçimi bu adımdan çıkarır. Sonuç önce en az `-1` yapılır, sonra pozitif limit ile döngüsel aralığa taşınır.

Rust karşılığı `crates/voxel-game/src/input/scroll.rs` içindeki `ScrollWheelHandler` tipidir. Java double-to-int dönüşümünün NaN/Infinity sonuçları ve i32 taşmalı selection arithmetic'i korunur; production callback ve player state entegrasyonu bu veri dilimine dahil değildir.

## JSONL sınırı

İstek `family: "math"`, `operation: "scroll_wheel"`, sıralı x/y scroll f64 bitleri ve wheel/current/limit selection sorgularını taşır. Sonuç her scroll çağrısının x/y tam adımını ve her selection sorgusunun giriş bitleri ile seçilen slotu taşır.


## Client-parity doğrulaması

`crates/voxel-game/src/input/scroll.rs`, Java Edition 26.2 / protokol 776 / Java 25 kaynak davranışına göre scroll accumulation ve döngüsel selection helper'ını uygular. Java oracle gerçek `ScrollWheelHandler` nesnesini ve static helper'ı çalıştırır; beklenen sonuç Rust adapter'ından türetilmez.

Math referansı `8cf7a43afb6a60906b47bfee079ed673741f5f86bab9c3cfb6e1c45e38c4349c`; tam corpus referansı `1e0328e7a1ac19c95185e78a0b5b06c565d34c401ad3477be3a28f22417094e0` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-aoa073hl` ve `compare-pid92coi`; tam debug/release karşılaştırmaları `compare-rvsvqgz1` ve `compare-53fxtari` raporlarında fark üretmedi.

Senaryolar fractional scroll birikimini, eksen yön değişiminde reset'i, iki eksenli tam adımları, signed zero/NaN/Infinity girdilerini, selection step yönlerini ve pozitif limitli slot sarma sınırlarını kapsar. MouseHandler callback/odak/frame yönlendirmesi, GUI scroll tüketimi ve player/spectator state güncellemeleri bu doğrulamanın kapsamı dışındadır.

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
