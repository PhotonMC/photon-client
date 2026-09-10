# P05.a — OptionInstance SliderableEnum doğrulaması

`src/option.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.SliderableEnum` davranışını uygular. Java oracle gerçek `GraphicsPreset` enum listesiyle oluşturulan `OptionInstance.SliderableEnum` nesnesini çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Math referansı `2cdf57ae10115e268494b63af3001a2c44c2a8937a82eb6729e9aeafbbca2708`; tam corpus referansı `e5908239acaab4eac19ee620d9ff2f10f148394cdf705931251d402258b24b1d` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-q6r7l613` ve `compare-_o1el210`; tam debug/release karşılaştırmaları `compare-ujhol5cm` ve `compare-1qbui8nf` raporlarında fark üretmedi.

Senaryolar enum sırasını, her değerin slider konumunu, uçlarda next/previous davranışını, üyelik doğrulamasını ve negatif, sınır, sonsuzluk ve NaN slider değerlerini kapsar. Codec, seçenek dosyası, slider widget callback'leri ve grafik preset uygulama yan etkileri bu doğrulamanın kapsamı dışındadır.

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
