# P05.a — OptionInstance LazyEnum sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, değer listesi çalışma zamanında üretilen cycle button option value set davranışını kapsar. Genel `OptionInstance` value taşıması, Codec serileştirmesi, CycleButton olayları ve seçenek dosyası sonraki ayar/GUI dilimlerinde kalır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:356-366`, `client/Options.java:938-958` ve `client/gui/components/CycleButton.java:313-347` incelendi. Oracle gerçek `OptionInstance.LazyEnum<String>` nesnesini ve her çağrıda üretilen `CycleButton.ValueListSupplier` sonucunu çalıştırır.

- `valueListSupplier` çağrıldığında değer supplier'ı bir kez çalışır; CycleButton supplier'ı dönen listeyi immutable snapshot olarak hem default hem selected listeye verir.
- `valueListSupplier` yeniden çağrıldığında supplier yeniden çalışır; önceki snapshot yeni listeyle değiştirilmez.
- `validateValue`, ayrı verilen validation function sonucunu doğrudan taşır. Bu function mevcut değer listesine bağlı olmak zorunda değildir.
- CycleButton'ın tıklama, Shift ile ters yönde ilerleme, scroll ve modulo döngüsü bu dilimin dışındadır.

Rust karşılığı `crates/voxel-game/src/option.rs` içindeki `LazyEnum<T, Values, Validator>` tipidir. Supplier ve validation function generic closure olarak tutulur; listeler çağrı başına sahiplenilmiş değer olarak döner.

## JSONL sınırı

İstek `family: "math"`, `operation: "lazy_enum"`, ardışık supplier listelerini, geçerli değer listesini ve probe değerlerini taşır. Sonuç ilk ve ikinci supplier snapshot'larını ve her probe için validation sonucunu taşır.


## Client-parity doğrulaması

`crates/voxel-game/src/option.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.LazyEnum` davranışını uygular. Java oracle gerçek `LazyEnum` supplier, validation function ve `CycleButton.ValueListSupplier` yollarını çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Math referansı `98e05ab8593dd463cfcb291851a8d40a7811a0177c62b55ee18df108bb63a001`; tam corpus referansı `b61a7e72e2f39b12b39f624fd688f784b59e627eda5084fe25156cca0fbfb83c` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-nrf3ujvg` ve `compare-xp6rhro8`; tam debug/release karşılaştırmaları `compare-5ltllvat` ve `compare-7fl9g6ro` raporlarında, Java tekrar doğrulaması `verify-r_rotijy` raporunda fark üretmedi. Senaryolar supplier'ın çağrı başına liste değiştirmesini, immutable snapshot sırasını ve liste dışındaki bir değeri kabul edebilen bağımsız validation function'ı kapsar.

CycleButton'ın gerçek GUI olayları, option callback'leri, Codec ve seçenek dosyası bu doğrulamanın kapsamı dışındadır.
