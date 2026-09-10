# P05.b — CharacterEvent sözleşmesi

Bu dilim, GLFW karakter callback'inden gelen tek codepoint değerinin Java istemci yardımcılarına aktarılmasını kapsar. Pencere callback sırası, text field düzenleme, font ölçümü ve IME pre-edit yönlendirmesi sonraki input/text/GUI dilimlerinde kalır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/input/CharacterEvent.java:5-12` ve `util/StringUtil.java:62-64` incelendi. Oracle, aynı codepoint üzerinde gerçek Java `CharacterEvent` metodlarını çalıştırır.

- `codepointAsString`, geçerli BMP codepoint'lerini tek UTF-16 code unit olarak, supplementary codepoint'lerini surrogate pair olarak taşır. Java'nın geçerli saydığı surrogate codepoint'leri de tek code unit olarak korunur; geçersiz aralık exception yoludur.
- `isAllowedChatCharacter`, codepoint'in en az 32 olmasını, DEL (127) ve section sign (167) olmamasını ister. Bu kontrol Unicode kategori veya grapheme çözümlemesi yapmaz.

Rust karşılığı `src/input_event.rs` içindeki `CharacterEvent` tipidir. Java string'in UTF-16 içeriği `codepoint_as_utf16` ile `Vec<u16>` olarak sunulur; Java'nın geçersiz codepoint exception'ı `InvalidCodePoint` sonucu ile temsil edilir.

## JSONL sınırı

İstek `family: "math"`, `operation: "character_event"` ve i32 `codepoint` alanını taşır. Sonuç codepoint'i, Java string'in UTF-16 code unit'lerini dört haneli hex alanları olarak ve chat karakteri izin sonucunu taşır.
