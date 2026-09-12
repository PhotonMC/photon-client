# P05.b — ToggleKeyMapping sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, temel `KeyMapping` tablosu üstündeki Java toggle davranışını kapsar. Gerçek ekranın key release akışı ve pencere odağı sonraki input/GUI dilimlerinde kalır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/ToggleKeyMapping.java:7-59` ile `client/KeyMapping.java:47-75` incelendi.

- `needsToggle` true olduğunda `setDown(true)` mevcut basılı durumun tersini kurar; `setDown(false)` durumu değiştirmez. `needsToggle` false olduğunda iki çağrı doğrudan verilen durumu taşır.
- `releaseAll`, toggle açıkken basılı olan mapping'in ekran tarafından bırakıldığını kaydeder ve durumu kapatır; click kuyruğu da temel mapping gibi temizlenir.
- `shouldRestoreStateOnScreenClosed`, `shouldRestore` true, key türü keysym ve kayıtlı geçici bırakma durumu birlikte sağlanırsa bir kez true döner; çağrı bu kaydı temizler. `restoreToggleStatesOnScreenClosed` bu sonucu alan mapping'i yeniden basılı duruma getirir.
- `resetToggleKeys` bütün toggle mapping'lerini kapatır. Toggle sınıfının `needsToggle` false olması, mapping'in toggle ailesine ait olmasını değiştirmez.

Rust karşılığı `src/key_mapping.rs` içindeki toggle davranışıdır. `KeyMappingRegistry::register_toggle`, `release_all`, `restore_toggle_states_on_screen_closed` ve `reset_toggle_keys` statik Java yollarının registry karşılıklarıdır.

## JSONL sınırı

İstek `family: "math"`, `operation: "toggle_key_mapping"`, üç mapping'in key bilgileri, toggle/restore bayrakları ve kontrollü action dizisini taşır. Sonuç başlangıç ve action sonrası basılı durumları, click tüketimlerini, tek seferlik restore sonuçlarını ve mapping özelliklerini taşır.


## Client-parity doğrulaması

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
