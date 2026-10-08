# P05.a — OptionInstance AltEnum sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, `OptionInstance.AltEnum` value set'inin çalışma anındaki koşula göre varsayılan veya alternatif değer listesini seçmesini kapsar. Genel `OptionInstance` value taşıması, özel value setter, Codec serileştirmesi, CycleButton olayları ve seçenek dosyası bu dilimin dışındadır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:169-185` ve `client/gui/components/CycleButton.java:313-347` incelendi. `AltEnum` şu davranışları taşır:

- Varsayılan ve alternatif listeler constructor sırasını ve duplicate değerleri korur.
- `valueListSupplier` default listeyi her zaman varsayılan değerlerden üretir; selected liste her çağrıda koşulu okuyarak varsayılan veya alternatif kopyayı verir.
- `validateValue`, koşul false iken varsayılan listede, true iken alternatif listede `contains` araması yapar ve bulunan değeri geri verir.
- Koşul çağrılarının sırası gözlenebilir; selected liste ve validation çağrıları ayrı koşul okumalarıdır.

Rust karşılığı `crates/photon-game/src/options/values.rs` içindeki `AltEnum<T, Condition>` tipidir. Koşul generic closure olarak tutulur; listeler sahiplenilmiş kopyalar olarak dışarı verilir.

## JSONL sınırı

İstek `family: "math"`, `operation: "alt_enum"`, iki ordered string listesini, koşul sonuçlarını ve validation probe değerlerini taşır. Sonuç varsayılan/alternatif listeleri, her selected-list çağrısının çıktısını ve probe validation sonuçlarını taşır.


## Client-parity doğrulaması

`crates/photon-game/src/options/values.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.AltEnum` value list ve koşullu validation davranışını uygular. Java oracle gerçek `OptionInstance.AltEnum<String>` ve `CycleButton.ValueListSupplier` yollarını çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Koşul değişimleri, duplicate değerler, boş alternatif liste ve listedeki/liste dışındaki probe değerleri sentetik senaryolarla karşılaştırılır. Özel `ValueSetter`, Codec, CycleButton'ın gerçek GUI olayları ve seçenek dosyası bu doğrulamanın kapsamı dışındadır.

Math referansı `8673a36cbe702888236f26a5a45a57c676dadb291b545d6eb5fc1044d5be1d8d`; tam corpus referansı `b36e2d451f7423a527e40a0ecf456fe2349ba13d5fb015c96af569a7dcc8ed72` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-uzy8bi1g` ve `compare-59zvt9rk`; tam debug/release karşılaştırmaları `compare-aio257r6` ve `compare-gq3enkyo` raporlarında, Java tekrar doğrulaması `verify-itxepyqo` raporunda fark üretmedi.

Tekrarlanabilir doğrulama komutları:

```text
python3 tools/behavior/run.py verify
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```
