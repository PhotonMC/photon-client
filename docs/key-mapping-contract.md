# P05.b — KeyMapping sözleşmesi

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
