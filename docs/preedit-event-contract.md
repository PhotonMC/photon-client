# P05.b — PreeditEvent callback sözleşmesi

Bu dilim, IME pre-edit callback'inden gelen codepoint ve block-size dizilerinin Java `PreeditEvent` verisine çevrilmesini kapsar. Native pointer yaşam döngüsü, GUI'ye event yönlendirmesi, component styling, caret çizimi ve IME platform entegrasyonu sonraki text/GUI dilimlerinde kalır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/input/PreeditEvent.java:14-83` incelendi. Oracle, kontrollü native bellekten gerçek Java `PreeditEvent.createFromCallback` yolunu çalıştırır; native bellek yalnızca test glue tarafından hazırlanır.

- `preeditSize` sıfırsa callback pointer'ları okunmadan null sonuç döner. Sıfırdan büyük preedit'te codepoint dizisi ve block-size dizisi verilen sırayla taranır.
- Her block size için Java döngüsünün gözlenen davranışı korunur: negatif boyut boş block üretir. Block'lar birleştirilirken codepoint'ler Java UTF-16 code unit'lerine çevrilir; supplementary karakterler iki unit kaplar.
- Caret, callback'in codepoint offset'i olarak okunur. Her codepoint öncesinde bulunduğu block'ın birikmiş UTF-16 uzunluğuna çevrilir; offset tüm preedit'in sonunda ise full text UTF-16 uzunluğu kullanılır. Hiçbir eşleşme yoksa Java başlangıç değeri korunur.
- Constructor focused block indeksini `[0, block sayısı)` aralığında ister. `toFormattedText` sonucu Component/Style katmanına bağlı olduğundan bu headless veri sözleşmesine dahil değildir.

Rust karşılığı `src/input_event.rs` içindeki `PreeditEvent` tipidir. Java `String` alanları, unpaired/supplementary UTF-16 içeriğini kaybetmemek için `Vec<u16>` ve `Vec<Vec<u16>>` olarak taşınır; callback'in null sonucu `None`, constructor ve veri hataları `PreeditError` ile temsil edilir.

## JSONL sınırı

İstek `family: "math"`, `operation: "preedit_event"`, codepoint dizisi, block-size dizisi, focused block ve caret alanlarını taşır. Sonuç boş callback'te Java JSON serileştirmesindeki boş nesneyi; event üretildiğinde full text, UTF-16 caret, block listesi ve focused block alanlarını taşır.
