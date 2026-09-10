# P05.b — PreeditEvent callback doğrulaması

`src/input_event.rs`, Java Edition 26.2 / protokol 776 / Java 25 kaynak davranışına göre IME pre-edit callback verisini, UTF-16 text block'larını ve caret dönüşümünü uygular. Java oracle, gerçek `PreeditEvent.createFromCallback` yolunu çalıştırır; native pointer belleği yalnızca test glue tarafından hazırlanır ve beklenen sonuç Rust çıktısından türetilmez.

Math referansı `7718a9a86f52fa96b5a5f104ce7b26e5db72febf33c9abc5550495ccf418a797`; tam corpus referansı `66fbb3e93e420267b5a728e46c1924b78feab7af86f879c6d78d3fa76be110b8` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-xw9ri3c2` ve `compare-ohe3yaqh`; tam debug/release karşılaştırmaları `compare-ijb41va0` ve `compare-h0_y8ft2` raporlarında fark üretmedi.

Senaryolar boş pre-edit null yolu, tekli ve çoklu block birleştirmesini, supplementary codepoint'in UTF-16 caret etkisini, block sınırındaki ve sondaki caret'i, negatif block size'ın boş block davranışını ve kullanılmayan artakalan codepoint'leri kapsar. Geçersiz pointer/bellek, focused block exception, native GLFW callback yaşam döngüsü, Component/Style formatlaması ve GUI/IME tüketimi bu dilimin kapsamı dışındadır.

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
