# P05.a — OptionInstance Enum sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, cycle button seçeneklerinin ordered value list ve üyelik doğrulaması davranışını kapsar. Genel `OptionInstance` value taşıması, Codec serileştirmesi, CycleButton tıklama/scroll akışı ve seçenek dosyası sonraki ayar/GUI dilimlerinde kalır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:224-265` ve `client/gui/components/CycleButton.java:313-347` incelendi. Oracle gerçek `OptionInstance.Enum<String>` nesnesini ve onun `CycleButton.ValueListSupplier` sonucunu çalıştırır.

- `Enum` value set'i constructor'a verilen liste sırasını korur.
- `valueListSupplier`, cycle button için aynı sıradaki immutable değer listesini hem default hem selected liste olarak sunar; duplicate değerler korunur.
- `validateValue`, listede `contains` ile bulunan değeri geri verir; listede olmayan değer boş sonuç üretir.
- CycleButton'ın tıklama, Shift ile ters yönde ilerleme, scroll ve modulo döngüsü bu dilimin dışındadır.

Rust karşılığı `crates/voxel-game/src/options/values.rs` içindeki `CycleableEnum<T>` tipidir. Değerler clone edilerek gözlemlenir; value listesi mutasyona açık bir referans olarak dışarı verilmez.

## JSONL sınırı

İstek `family: "math"`, `operation: "cycleable_enum"`, ordered string `values` listesini ve membership probe değerlerini taşır. Sonuç default/selected value listelerini ve her probe için doğrulama sonucunu taşır.


## Client-parity doğrulaması

`crates/voxel-game/src/options/values.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.Enum` value list ve validation davranışını uygular. Java oracle gerçek Java `OptionInstance.Enum<String>` ve `CycleButton.ValueListSupplier` yollarını çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Math referansı `5aea1312d36c1500e8c18430f34e9c7d2debdb0ab3bf52d6adf7ec327b31aeef`; tam corpus referansı `46b1b9991e2bbfe2deb0ab630626cd047fa111697cf78408552a34159fed0266` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-k5s4yghv` ve `compare-pge6l_6l`; tam debug/release karşılaştırmaları `compare-c8vviwok` ve `compare-yvvp73ji` raporlarında, Java tekrar doğrulaması `verify-cg051ddh` raporunda fark üretmedi.

CycleButton'ın gerçek GUI olayları, tıklama/scroll modulo döngüsü, option callback'leri, Codec ve seçenek dosyası bu doğrulamanın kapsamı dışındadır.
