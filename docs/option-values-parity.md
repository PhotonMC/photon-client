# P05.a — OptionInstance value set doğrulaması

`src/option.rs`, Java Edition 26.2 / protokol 776 / Java 25 kaynak davranışına göre `OptionInstance.IntRange` ve `OptionInstance.UnitDouble` value set yollarını uygular. Java oracle gerçek Java value set nesnelerini çalıştırır; beklenen sonuç Rust adapter'ından türetilmez.

Math referansı `6228175f582cf55852ca8f04ace79e1f036490bfbfd8c7ad0dd2b4963509b616`; tam corpus referansı `cd902ce59c38f00873b5de59fc904974ee173bfc8a4027b93c5ecfb024f28eec` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-8nolf8i2` ve `compare-utpldtzr`; tam debug/release karşılaştırmaları `compare-1nsattlt` ve `compare-0zjc7zb0` raporlarında fark üretmedi.

Senaryolar pozitif/negatif IntRange'leri, aralık dışı integer doğrulamasını, next/previous sınırlarını, slider sınırlarını ve özel f64 değerlerini, i32 üst sınırına yakın aralığı ve UnitDouble'ın signed zero dahil geçerli/geçersiz değerlerini kapsar. Option codec'leri, option dosyası, slider widget callback'leri ve GUI davranışı bu doğrulamanın kapsamı dışındadır.

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
