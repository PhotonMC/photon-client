# P05.a — OptionInstance value set sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, ayar widget'ı veya dosya yükleme gerektirmeyen `IntRange` ve `UnitDouble` value set davranışını kapsar. OptionInstance'ın genel value taşıması, Codec JSON serileştirmesi, seçenek dosyası ve GUI callback akışı sonraki ayar/GUI dilimlerinde kalır.

## Kaynak ve davranış

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `client/OptionInstance.java:267-353`, `client/OptionInstance.java:543-623` ve `util/Mth.java:344-349,558-560` incelendi. Oracle, gerçek Java value set nesneleri üzerinde doğrulama ve slider dönüşümlerini çalıştırır.

- `IntRange`, min/max uçlarını kapsayan bir aralık doğrular. `next` ve `previous` doğrudan Java i32 toplama/çıkarma yollarıdır; aralık dışına çıkabilir ve i32 taşması sarılır.
- `toSliderValue`, min için `0.0`, max için `1.0` döndürür; diğer değerleri yarım hücre ofsetiyle `Mth.map` yolundan geçirir.
- `fromSliderValue`, slider `1.0` veya üstündeyse Java f32 `0.99999` değerini kullanır; ardından map sonucunu `floor` ile i32'ye çevirir. NaN, sonsuzluk ve aralık dışı slider değerleri bu dönüşüm yolunun parçasıdır.
- `UnitDouble`, `0.0 <= value <= 1.0` koşulunu doğrular. Slider'a çevirme ve slider'dan dönme değeri bit düzeyinde değiştirmez; NaN doğrulamadan geçmez.

Rust karşılığı `crates/voxel-game/src/option.rs` içindeki `IntRange` ve `UnitDouble` tipleridir. `IntRange::with_apply_value_immediately`, Java record alanını taşır; codec, xmap ve GUI/widget yöntemleri bu veri dilimine dahil değildir.

## JSONL sınırı

İstek `family: "math"`, `operation: "option_values"`, IntRange uçlarını/uygulama bayrağını, integer değerlerini, slider f64 bitlerini ve UnitDouble f64 bitlerini taşır. Sonuç doğrulama, next/previous, slider dönüşümü ve UnitDouble kimlik yollarını f64 bitleriyle taşır.


## Client-parity doğrulaması

`crates/voxel-game/src/option.rs`, Java Edition 26.2 / protokol 776 / Java 25 kaynak davranışına göre `OptionInstance.IntRange` ve `OptionInstance.UnitDouble` value set yollarını uygular. Java oracle gerçek Java value set nesnelerini çalıştırır; beklenen sonuç Rust adapter'ından türetilmez.

Math referansı `6228175f582cf55852ca8f04ace79e1f036490bfbfd8c7ad0dd2b4963509b616`; tam corpus referansı `cd902ce59c38f00873b5de59fc904974ee173bfc8a4027b93c5ecfb024f28eec` olarak kaydedildi. Math debug/release karşılaştırmaları `compare-8nolf8i2` ve `compare-utpldtzr`; tam debug/release karşılaştırmaları `compare-1nsattlt` ve `compare-0zjc7zb0` raporlarında fark üretmedi.

Senaryolar pozitif/negatif IntRange'leri, aralık dışı integer doğrulamasını, next/previous sınırlarını, slider sınırlarını ve özel f64 değerlerini, i32 üst sınırına yakın aralığı ve UnitDouble'ın signed zero dahil geçerli/geçersiz değerlerini kapsar. Option codec'leri, option dosyası, slider widget callback'leri ve GUI davranışı bu doğrulamanın kapsamı dışındadır.

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
