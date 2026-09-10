# P05.b — InputWithModifiers ve event kayıtları sözleşmesi

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

Rust karşılığı `src/input_event.rs` içindeki `InputWithModifiers`, `KeyEvent`, `MouseButtonInfo` ve `MouseButtonEvent` tipleridir. `InputQuirks` Linux oracle ortamının sabitlerini taşır; adapter sonucu Java oracle şemasıyla karşılaştırır.
