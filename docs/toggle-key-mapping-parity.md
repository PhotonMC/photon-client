# P05.b — ToggleKeyMapping doğrulaması

`src/key_mapping.rs`, Java Edition 26.2 / protokol 776 / Java 25 `ToggleKeyMapping` davranışını temel `KeyMapping` tablosuyla birlikte uygular. Oracle, gerçek toggle alt sınıfı üzerinde tekrar basma, direct-set, `releaseAll`, ekran kapanışında geri yükleme ve reset yollarını çalıştırır; beklenen sonuç Rust çıktısından türetilmez.

Math referansı `aadf293a10f1b023a7658ef9bd32adbc3eb682124febea5ffc45cec36825aca8`; tam corpus referansı `35a15ff319bf9648f521faf427804b03cdbf8490a89ab9d6bebcf102169e9837` olarak kaydedildi. Debug karşılaştırması `compare-nfctje7g`, release karşılaştırması `compare-zaxnxs7j`, Java tekrar doğrulaması `verify-badjzt7s` raporlarında fark üretmedi.

Senaryolar toggle açıkken `setDown(true)` terslemesini, aynı key’e bağlı toggle fan-out’unu, `needsToggle` false direct-set yolunu, keysym/scancode restore ayrımını, restore kaydının tek kullanımlı olmasını ve `resetToggleKeys` temizliğini kapsar. Gerçek ekran kapanışı, GLFW odağı ve `KeyboardHandler` yönlendirmesi sonraki input/GUI dilimlerindedir.

Tekrarlanabilir doğrulama komutları:

```text
python3 tools/behavior/run.py verify
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --adapter target/release/examples/behavior_adapter
python3 tools/behavior/test_runner.py
cargo test --locked --offline --all-targets
cargo clippy --locked --offline --all-targets --all-features -- -D warnings
```
