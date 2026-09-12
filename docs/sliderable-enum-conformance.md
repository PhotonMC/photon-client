# P05.a — OptionInstance SliderableEnum sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, ayar widget'ı veya seçenek dosyası gerektirmeyen sıralı enum value set davranışını kapsar. Genel `OptionInstance` value taşıması, Codec JSON serileştirmesi ve GUI callback akışı sonraki ayar/GUI dilimlerinde kalır.

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:484-522`, `client/Options.java:157-165`, `client/GraphicsPreset.java:12-20` ve `util/Mth.java:344-349,558-560` incelendi. Oracle gerçek `OptionInstance.SliderableEnum<GraphicsPreset>` nesnesini çalıştırır.

- Değer sırası `GraphicsPreset.values()` sırasıdır: `FAST`, `FANCY`, `FABULOUS`, `CUSTOM`.
- İlk değer slider `0.0`, son değer slider `1.0` alır. Diğer değerler enum indeksini `Mth.map` ile `[0, 1]` aralığına taşır.
- `next` ve `previous` mevcut indeksin bir üstünü veya altını seçer; uçlarda indeks `[0, values.size() - 1]` içinde tutulur.
- `fromSliderValue`, slider `1.0` veya üstündeyse Java f32 `0.99999` değerini kullanır. Ardından `Mth.map` sonucu `floor` edilir ve enum indeksine sıkıştırılır. Negatif, sonsuz ve NaN slider değerleri aynı Java dönüşüm yolundadır.
- `validateValue`, değerin value listesinde bulunup bulunmadığını döndürür.

Rust karşılığı `src/option.rs` içindeki `SliderableEnum<T>` tipidir. Value listesi boş olamaz; Java'da bu record ile oluşturulabilse de slider metotları boş listede ilk/son değere eriştiği için bu dilim istemcide kullanılan geçerli enum listelerini modeller.

## JSONL sınırı

İstek `family: "math"`, `operation: "sliderable_enum"`, probe enum adlarını ve slider f64 bitlerini taşır. Sonuç value listesi, her probe için doğrulama/slider/komşu enum değerlerini ve her slider için `fromSliderValue` enum sonucunu taşır.


## Client-parity doğrulaması

`src/option.rs`, Java Edition 26.2 / protokol 776 / Java 25 `OptionInstance.SliderableEnum` davranışını uygular. Java oracle gerçek `GraphicsPreset` enum listesiyle oluşturulan `OptionInstance.SliderableEnum` nesnesini çalıştırır; beklenen sonuç Rust adaptöründen türetilmez.

Math referansı `2cdf57ae10115e268494b63af3001a2c44c2a8937a82eb6729e9aeafbbca2708`; tam corpus referansı `e5908239acaab4eac19ee620d9ff2f10f148394cdf705931251d402258b24b1d` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-q6r7l613` ve `compare-_o1el210`; tam debug/release karşılaştırmaları `compare-ujhol5cm` ve `compare-1qbui8nf` raporlarında fark üretmedi.

Senaryolar enum sırasını, her değerin slider konumunu, uçlarda next/previous davranışını, üyelik doğrulamasını ve negatif, sınır, sonsuzluk ve NaN slider değerlerini kapsar. Codec, seçenek dosyası, slider widget callback'leri ve grafik preset uygulama yan etkileri bu doğrulamanın kapsamı dışındadır.

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
