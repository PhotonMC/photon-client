# P05.b — KeyMapping sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java istemcisindeki temel `KeyMapping` durumunu ve pencere olayından önceki key binding tablosunu kapsar. `ToggleKeyMapping` davranışı, GLFW callback kurulumu, `KeyboardHandler` içindeki ekran/oyun yönlendirmesi ve gerçek pencere odağı ayrı dilimlerdir.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/KeyMapping.java:24-218` ve `com/mojang/blaze3d/platform/InputConstants.java:237-342` incelendi.

- Bir key mapping geçerli key'ini ve oluşturulduğu default key'i saklar. Key kimliği tür ve sayısal değerden oluşur: keysym, scancode veya mouse.
- Oluşturma, mapping'i geçerli key altında ortak statik tabloya ekler. Aynı key'e bağlı mapping'lere `set` basılı durumu, `click` ise bağımsız click kuyruğu girdisi olarak birlikte uygulanır.
- `consumeClick` kuyruk boşsa `false`, doluysa tek girdiyi tüketerek `true` döndürür. `releaseAll` bütün mapping'lerin click kuyruğunu ve basılı durumunu temizler.
- `setKey` mapping'in geçerli key'ini değiştirir; Java statik binding tablosunu yeniden kurmadığı için eski key üzerinden gelen statik olaylar mapping'i etkilemeye devam eder.
- `matches(KeyEvent)`, bilinmeyen key olayında scancode türünü; normal key olayında keysym türünü arar. `matchesMouse` yalnızca mouse türünü kabul eder. `same`, key türü ve değer eşitliğini kullanır.
- `isDefault`, `isUnbound` ve `saveString` geçerli key ile default/unknown durumunu ve serialized key adını gözlemlenebilir kılar.

Rust karşılığı `src/key_mapping.rs` içindeki `InputKey`, `KeyMapping` ve `KeyMappingRegistry` tipleridir. Registry'deki key tablosu `setKey` sonrasında bilerek yeniden kurulmaz.

## JSONL sınırı

İstek `family: "math"`, `operation: "key_mapping"` ve üç mapping tanımı, bir rebind key'i, kontrollü action dizisi ile key/mouse probe değerlerini taşır. Sonuç başlangıç ve action sonrası basılı durumlarını; click tüketimlerini; key adı, default, unbound, save ve event eşleşmelerini taşır.


## Client-parity doğrulaması

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
