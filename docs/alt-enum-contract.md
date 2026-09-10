# P05.a — OptionInstance AltEnum sözleşmesi

Bu dilim, `OptionInstance.AltEnum` value set'inin çalışma anındaki koşula göre varsayılan veya alternatif değer listesini seçmesini kapsar. Genel `OptionInstance` value taşıması, özel value setter, Codec serileştirmesi, CycleButton olayları ve seçenek dosyası bu dilimin dışındadır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:169-185` ve `client/gui/components/CycleButton.java:313-347` incelendi. `AltEnum` şu davranışları taşır:

- Varsayılan ve alternatif listeler constructor sırasını ve duplicate değerleri korur.
- `valueListSupplier` default listeyi her zaman varsayılan değerlerden üretir; selected liste her çağrıda koşulu okuyarak varsayılan veya alternatif kopyayı verir.
- `validateValue`, koşul false iken varsayılan listede, true iken alternatif listede `contains` araması yapar ve bulunan değeri geri verir.
- Koşul çağrılarının sırası gözlenebilir; selected liste ve validation çağrıları ayrı koşul okumalarıdır.

Rust karşılığı `src/option.rs` içindeki `AltEnum<T, Condition>` tipidir. Koşul generic closure olarak tutulur; listeler sahiplenilmiş kopyalar olarak dışarı verilir.

## JSONL sınırı

İstek `family: "math"`, `operation: "alt_enum"`, iki ordered string listesini, koşul sonuçlarını ve validation probe değerlerini taşır. Sonuç varsayılan/alternatif listeleri, her selected-list çağrısının çıktısını ve probe validation sonuçlarını taşır.
