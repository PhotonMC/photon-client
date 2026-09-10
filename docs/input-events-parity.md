# P05.b — InputWithModifiers ve event kayıtları doğrulaması

`src/input_event.rs`, Java Edition 26.2 / protokol 776 / Java 25 kaynak davranışına göre ortak input sınıflandırmalarını, modifier yardımcılarını ve keyboard/mouse event kayıtlarını uygular. Java oracle aynı değerleri gerçek `InputWithModifiers` implementasyonları üzerinden çalıştırır; beklenen sonuç Rust çıktısından türetilmez.

Math referansı `340fc7a9577da0828f076157cecb5cc720483ae3bec7b7728073e666b0618a6a`; tam corpus referansı `19a0ba84836a84126f94a904aff0246e026161812bc12ae9380ea8b65a00475f` olarak kaydedildi. Math debug/release karşılaştırmaları sırasıyla `compare-5g4a7a0d` ve `compare-98z0c0tm`; tam debug/release karşılaştırmaları `compare-521vx__1` ve `compare-rzccakse`; Java tekrar doğrulaması `verify-47v4glid` raporlarında fark üretmedi.

Senaryolar seçim/onay tuşlarını, escape/yön/focus sınıflandırmalarını, digit sınırlarını ve i32 alt taşmasını, modifier bitlerini, Linux Ctrl kısayollarını, Shift/Alt engellerini, üç event record'unun alanlarını ve signed-zero koordinatlarını kapsar. Gerçek pencere callback'leri, odak kaybı, mouse grab, metin/IME ve oynanabilir istemci bağlantısı bu doğrulamanın kapsamı dışındadır.

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
