# P05.b — InputWithModifiers ve event kayıtları sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, pencere callback'lerine ulaşan temel keyboard/mouse event değerlerinin ortak Java yardımcılarını kapsar. GLFW callback sırası, odak değişimi, tuş tekrarı, gerçek ekran yönlendirmesi ve metin/IME düzenleme akışı sonraki input/GUI dilimlerinde kalır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/input/InputWithModifiers.java:9-89`, `client/input/InputQuirks.java:3-8`, `client/input/KeyEvent.java:9-20`, `client/input/MouseButtonInfo.java:8-23` ve `client/input/MouseButtonEvent.java:3-18` incelendi. Oracle, aynı input ve modifier değerlerini üç Java record'u üzerinde çalıştırır.

- `isSelection`, Enter, keypad Enter ve Space değerlerini; `isConfirmation`, Enter ve keypad Enter değerlerini tanır. Escape, dört ok ve focus döngüsü ayrı sabit input değerleriyle sınıflandırılır.
- `getDigit`, input değerinden ASCII `'0'` farkını Java i32 aritmetiğiyle hesaplar; sonuç 0–9 dışındaysa `NOT_DIGIT` döner. Bu nedenle i32 alt sınırı gibi taşma girdileri de sözleşmenin parçasıdır.
- Modifier maskesi Shift, Control, Alt ve Super bitlerini ayrı okur. Linux referansında düzenleme kısayolu modifier'ı Control'dür; select-all/copy/paste/cut yalnızca ilgili harf, Control ve modifier'sız Shift/Alt koşulları birlikte sağlandığında true olur.
- `KeyEvent` key, scancode ve modifier değerlerini; `MouseButtonInfo` button ve modifier değerlerini taşır. `MouseButtonEvent`, pencere x/y koordinatlarını ve button bilgisini taşır; ortak helper'lar mouse button üzerinden çalışır.
- `InputQuirks` içindeki bu dilimde kullanılan Linux davranışı, Ctrl yerine Command eşlemesini ve uzun sol tıkla sağ tık benzetimini kapalı, mouse grab sonrasında key state geri yüklemesini açık tutar. Bu sabitlerin callback akışı üzerindeki etkisi ileride ayrıca ölçülür.

## JSONL sınırı

İstek `family: "math"`, `operation: "input_with_modifiers"`, keyboard key/scancode, mouse button, modifier maskesi ve f64 bitleriyle verilen x/y koordinatlarını taşır. Sonuç her üç Java record'u için ortak helper alanlarını; ayrıca record'a özgü key, scancode, button ve koordinat bitlerini taşır.

Rust karşılığı `crates/voxel-game/src/input_event.rs` içindeki `InputWithModifiers`, `KeyEvent`, `MouseButtonInfo` ve `MouseButtonEvent` tipleridir. `InputQuirks` Linux oracle ortamının sabitlerini taşır; adapter sonucu Java oracle şemasıyla karşılaştırır.


## Client-parity doğrulaması

`crates/voxel-game/src/input_event.rs`, Java Edition 26.2 / protokol 776 / Java 25 kaynak davranışına göre ortak input sınıflandırmalarını, modifier yardımcılarını ve keyboard/mouse event kayıtlarını uygular. Java oracle aynı değerleri gerçek `InputWithModifiers` implementasyonları üzerinden çalıştırır; beklenen sonuç Rust çıktısından türetilmez.

Math referansı `340fc7a9577da0828f076157cecb5cc720483ae3bec7b7728073e666b0618a6a`; tam corpus referansı `19a0ba84836a84126f94a904aff0246e026161812bc12ae9380ea8b65a00475f` olarak kaydedildi. Math debug/release karşılaştırmaları sırasıyla `compare-5g4a7a0d` ve `compare-98z0c0tm`; tam debug/release karşılaştırmaları `compare-521vx__1` ve `compare-rzccakse`; Java tekrar doğrulaması `verify-47v4glid` raporlarında fark üretmedi.

Senaryolar seçim/onay tuşlarını, escape/yön/focus sınıflandırmalarını, digit sınırlarını ve i32 alt taşmasını, modifier bitlerini, Linux Ctrl kısayollarını, Shift/Alt engellerini, üç event record'unun alanlarını ve signed-zero koordinatlarını kapsar. Gerçek pencere callback'leri, odak kaybı, mouse grab, metin/IME ve oynanabilir istemci bağlantısı bu doğrulamanın kapsamı dışındadır.

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
