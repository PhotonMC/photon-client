# P05.c — KeyboardInput doğrulaması

`src/input.rs`, Java Edition 26.2 / protokol 776 / Java 25 kaynak davranışına göre `KeyboardInput.tick()` snapshot’ını ve normalize edilmiş hareket vektörünü uygular. Java oracle, kontrollü `KeyMapping` durumlarıyla gerçek `KeyboardInput.tick()` çağrısını çalıştırır; beklenen sonuç Rust adapter’ından türetilmez.

Java referansı `b8ac51bd8c3bd440346514a4021569847347a2d7fedd0409b4fc84e14c8cc9f6` capture’ında iki tekrarla kararlı kaldı. Debug adapter karşılaştırması `compare-058fr0qu`, release adapter karşılaştırması `compare-_6wiu7ho` ve tam Java doğrulaması `verify-j9ido3zp` raporlarında eşleşti.

Doğrulanan senaryolar karşıt yönlerin sıfırlanmasını, tek ve çapraz hareketleri, dört hareket yönünü birlikte, jump/shift/sprint alanlarını ve hareket bayraklarıyla birlikte snapshot üretimini kapsar. Gerçek pencere olayları, GLFW odak geçişleri ve tuş tekrarları P05.b/d kapsamındadır; oynanabilir istemci bağlantısı henüz yoktur.

Tekrarlanabilir doğrulama komutları:

```text
python3 tools/behavior/run.py verify
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --adapter target/release/examples/behavior_adapter
python3 tools/behavior/test_runner.py
cargo test --locked --offline --all-targets
cargo clippy --locked --offline --all-targets --all-features -- -D warnings
```
