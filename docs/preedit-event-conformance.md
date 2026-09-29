# P05.b — PreeditEvent callback sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, IME pre-edit callback'inden gelen codepoint ve block-size dizilerinin Java `PreeditEvent` verisine çevrilmesini kapsar. Native pointer yaşam döngüsü, GUI'ye event yönlendirmesi, component styling, caret çizimi ve IME platform entegrasyonu sonraki text/GUI dilimlerinde kalır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/input/PreeditEvent.java:14-83` incelendi. Oracle, kontrollü native bellekten gerçek Java `PreeditEvent.createFromCallback` yolunu çalıştırır; native bellek yalnızca test glue tarafından hazırlanır.

- `preeditSize` sıfırsa callback pointer'ları okunmadan null sonuç döner. Sıfırdan büyük preedit'te codepoint dizisi ve block-size dizisi verilen sırayla taranır.
- Her block size için Java döngüsünün gözlenen davranışı korunur: negatif boyut boş block üretir. Block'lar birleştirilirken codepoint'ler Java UTF-16 code unit'lerine çevrilir; supplementary karakterler iki unit kaplar.
- Caret, callback'in codepoint offset'i olarak okunur. Her codepoint öncesinde bulunduğu block'ın birikmiş UTF-16 uzunluğuna çevrilir; offset tüm preedit'in sonunda ise full text UTF-16 uzunluğu kullanılır. Hiçbir eşleşme yoksa Java başlangıç değeri korunur.
- Constructor focused block indeksini `[0, block sayısı)` aralığında ister. `toFormattedText` sonucu Component/Style katmanına bağlı olduğundan bu headless veri sözleşmesine dahil değildir.

Rust karşılığı `crates/voxel-game/src/input_event.rs` içindeki `PreeditEvent` tipidir. Java `String` alanları, unpaired/supplementary UTF-16 içeriğini kaybetmemek için `Vec<u16>` ve `Vec<Vec<u16>>` olarak taşınır; callback'in null sonucu `None`, constructor ve veri hataları `PreeditError` ile temsil edilir.

## JSONL sınırı

İstek `family: "math"`, `operation: "preedit_event"`, codepoint dizisi, block-size dizisi, focused block ve caret alanlarını taşır. Sonuç boş callback'te Java JSON serileştirmesindeki boş nesneyi; event üretildiğinde full text, UTF-16 caret, block listesi ve focused block alanlarını taşır.


## Client-parity doğrulaması

`crates/voxel-game/src/input_event.rs`, Java Edition 26.2 / protokol 776 / Java 25 kaynak davranışına göre IME pre-edit callback verisini, UTF-16 text block'larını ve caret dönüşümünü uygular. Java oracle, gerçek `PreeditEvent.createFromCallback` yolunu çalıştırır; native pointer belleği yalnızca test glue tarafından hazırlanır ve beklenen sonuç Rust çıktısından türetilmez.

Math referansı `7718a9a86f52fa96b5a5f104ce7b26e5db72febf33c9abc5550495ccf418a797`; tam corpus referansı `66fbb3e93e420267b5a728e46c1924b78feab7af86f879c6d78d3fa76be110b8` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-xw9ri3c2` ve `compare-ohe3yaqh`; tam debug/release karşılaştırmaları `compare-ijb41va0` ve `compare-h0_y8ft2` raporlarında fark üretmedi.

Senaryolar boş pre-edit null yolu, tekli ve çoklu block birleştirmesini, supplementary codepoint'in UTF-16 caret etkisini, block sınırındaki ve sondaki caret'i, negatif block size'ın boş block davranışını ve kullanılmayan artakalan codepoint'leri kapsar. Geçersiz pointer/bellek, focused block exception, native GLFW callback yaşam döngüsü, Component/Style formatlaması ve GUI/IME tüketimi bu dilimin kapsamı dışındadır.

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
