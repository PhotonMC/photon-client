# P05.b — ToggleKeyMapping sözleşmesi

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
