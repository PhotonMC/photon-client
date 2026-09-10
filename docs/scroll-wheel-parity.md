# P05.d — ScrollWheelHandler doğrulaması

`src/scroll.rs`, Java Edition 26.2 / protokol 776 / Java 25 kaynak davranışına göre scroll accumulation ve döngüsel selection helper'ını uygular. Java oracle gerçek `ScrollWheelHandler` nesnesini ve static helper'ı çalıştırır; beklenen sonuç Rust adapter'ından türetilmez.

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
