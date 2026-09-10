# P05.a — OptionInstance Enum sözleşmesi

Bu dilim, cycle button seçeneklerinin ordered value list ve üyelik doğrulaması davranışını kapsar. Genel `OptionInstance` value taşıması, Codec serileştirmesi, CycleButton tıklama/scroll akışı ve seçenek dosyası sonraki ayar/GUI dilimlerinde kalır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:224-265` ve `client/gui/components/CycleButton.java:313-347` incelendi. Oracle gerçek `OptionInstance.Enum<String>` nesnesini ve onun `CycleButton.ValueListSupplier` sonucunu çalıştırır.

- `Enum` value set'i constructor'a verilen liste sırasını korur.
- `valueListSupplier`, cycle button için aynı sıradaki immutable değer listesini hem default hem selected liste olarak sunar; duplicate değerler korunur.
- `validateValue`, listede `contains` ile bulunan değeri geri verir; listede olmayan değer boş sonuç üretir.
- CycleButton'ın tıklama, Shift ile ters yönde ilerleme, scroll ve modulo döngüsü bu dilimin dışındadır.

Rust karşılığı `src/option.rs` içindeki `CycleableEnum<T>` tipidir. Değerler clone edilerek gözlemlenir; value listesi mutasyona açık bir referans olarak dışarı verilmez.

## JSONL sınırı

İstek `family: "math"`, `operation: "cycleable_enum"`, ordered string `values` listesini ve membership probe değerlerini taşır. Sonuç default/selected value listelerini ve her probe için doğrulama sonucunu taşır.
