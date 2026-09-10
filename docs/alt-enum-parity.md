# P05.a — OptionInstance AltEnum doğrulaması

`src/option.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.AltEnum` value list ve koşullu validation davranışını uygular. Java oracle gerçek `OptionInstance.AltEnum<String>` ve `CycleButton.ValueListSupplier` yollarını çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Koşul değişimleri, duplicate değerler, boş alternatif liste ve listedeki/liste dışındaki probe değerleri sentetik senaryolarla karşılaştırılır. Özel `ValueSetter`, Codec, CycleButton'ın gerçek GUI olayları ve seçenek dosyası bu doğrulamanın kapsamı dışındadır.

Math referansı `8673a36cbe702888236f26a5a45a57c676dadb291b545d6eb5fc1044d5be1d8d`; tam corpus referansı `b36e2d451f7423a527e40a0ecf456fe2349ba13d5fb015c96af569a7dcc8ed72` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-uzy8bi1g` ve `compare-59zvt9rk`; tam debug/release karşılaştırmaları `compare-aio257r6` ve `compare-gq3enkyo` raporlarında, Java tekrar doğrulaması `verify-itxepyqo` raporunda fark üretmedi.

Tekrarlanabilir doğrulama komutları:

```text
python3 tools/behavior/run.py verify
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```
