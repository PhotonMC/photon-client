# P01.d — VoxelShape join ve joinUnoptimized sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `Shapes.join` sonucunun `optimize` adımından geçmesini, `Shapes.joinUnoptimized` sonucunun birleşik coordinate plane'lerini korumasını ve empty shape fast path'lerinin diğer shape'in koordinatlarını taşımasını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:135-173`, `VoxelShape.java:96-100,117-139` ve `DiscreteVoxelShape.java` içindeki box taraması incelendi. `MathOracle`, iki Java metodunu doğrudan çağırır ve her sonucun box, bounds ve `getCoords` değerlerini ham f64 bitleriyle kaydeder.

## Davranış sözleşmesi

- `Shapes.join(first, second, op)`, önce `joinUnoptimized` ile sonucu üretir, ardından `VoxelShape.optimize()` çağırır.
- `Shapes.joinUnoptimized`, BooleanOp sonucunu birleşik coordinate grid'inin her hücresine uygular; girdilerden biri empty ise `op` yalnızca diğer tarafı önemsiyorsa diğer shape'i, aksi halde empty shape'i döndürür.
- `joinUnoptimized` sonucu, dolu hücreler sonradan birleşebilse bile girdilerin coordinate plane'lerini korur. `optimize` box iterasyonu ile yeniden kurulduğundan touching kutular ortak plane'e indirgenebilir.
- Empty sonuçta Java'nın empty coordinate listeleri korunur; non-empty sonuçta koordinat düzlemleri ve box sırası gözlenebilir davranışın parçasıdır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `touching` | X ekseninde bitişen iki kutu, `OR` | `joinUnoptimized` ara `.25` plane'ini korur; `join` optimize edip `.5` sınırlarına iner |
| `hole` | Birim kutudan orta X dilimi çıkarılır, `ONLY_FIRST` | İki sonuç da iki dış kutuyu ve tüm birleşik coordinate plane'lerini taşır |
| `empty-first` | Sliver empty shape ile uzak bir kutu, `OR` | Her iki sonuç da ikinci shape'in koordinatlarını doğrudan taşır |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "join_coords"` biçimindedir. `unoptimized` ve `optimized` alanları `empty`, `boxes`, `bounds` ve `coords` snapshot'larını içerir.

## Java yakalama

Math referans kimliği `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` oldu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/photon-core/src/shape.rs`, Java `Shapes.join` çağrısının `joinUnoptimized(...).optimize()` akışını ve coordinate grid'ini koruyan `join_unoptimized` yolunu uygular. `examples/behavior_adapter.rs`, iki sonucu `getCoords` karşılıklarıyla JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `join_coords` senaryosunda ham ve optimize sonuçların box, bounds ve coordinate plane değerleri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu dilim üç sentetik box senaryosuyla sınırlıdır. Rust'ta Java'nın referans tür kimliği gözlemi değer semantiğine indirilmiştir; daha geniş index merger türleri ve istemci dünyasındaki shape toplama sonraki dilimlerdir.
