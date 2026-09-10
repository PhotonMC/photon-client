# P05.b — KeyMapping doğrulaması

`src/key_mapping.rs`, Java Edition 26.2 / protokol 776 / Java 25 `KeyMapping` ve `InputConstants.Key` davranışının pencere bağımsız alt kümesini uygular. Oracle, gerçek Java sınıflarını kullanarak ortak binding fan-out’unu, click kuyruğunu, release temizliğini, key türlerini, event eşleşmesini ve `setKey` sonrası eski statik tablo kaydını karşılaştırır.

Math referansı `578f8723ad1c7bc644cfc9bda4f953b98843ea1acde7a16ca037b5f26ee04ceb`; tam corpus referansı `c114281f9e92e9c6591468d78714c1e02163f2a1bd9ccd8b79fbbc8887b149cc` olarak kaydedildi. Debug karşılaştırması `compare-avq6g26v`, release karşılaştırması `compare-b0hxng2i`, Java tekrar doğrulaması `verify-eg54g67s` raporlarında fark üretmedi.

Senaryolar aynı key’e bağlı mapping’lerin birlikte güncellenmesini, press/repeat click tüketimini, `releaseAll`, keysym/scancode/mouse eşleşmelerini, default/unbound durumlarını ve yeniden binding’in eski tabloyu koruyan Java davranışını kapsar. `ToggleKeyMapping`, GLFW callback’leri ve `KeyboardHandler` ekran/oyun akışı sonraki pencere/input dilimlerindedir.

Tekrarlanabilir doğrulama komutları:

```text
python3 tools/behavior/run.py verify
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --adapter target/release/examples/behavior_adapter
python3 tools/behavior/test_runner.py
cargo test --locked --offline --all-targets
cargo clippy --locked --offline --all-targets --all-features -- -D warnings
```
