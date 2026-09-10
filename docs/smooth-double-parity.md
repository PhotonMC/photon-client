# P05.d — SmoothDouble doğrulaması

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
