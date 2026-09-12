# P01.d — Shapes.INFINITY sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `Shapes.INFINITY` sabitinin sınırsız tek shape olarak kurulmasını, box callback'ini ve aynı shape'in BooleanOp self sorgusunu içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:29-39`, `Shapes.java:44-52` ve `VoxelShape.java:117-139` incelendi. `MathOracle`, sabiti doğrudan kullanır; beklenen shape veya box listesi yeniden hesaplanmaz.

## Davranış sözleşmesi

- `Shapes.INFINITY`, her eksende `Double.NEGATIVE_INFINITY` ile `Double.POSITIVE_INFINITY` arasını dolduran tek hücreli shape'tir.
- `forAllBoxes` bu shape için bir callback çağrısı yapar ve altı sonsuz endpoint'i Java callback sırasıyla taşır.
- Aynı infinity shape iki operand olarak `NOT_SAME` ile birleştirildiğinde bütün hücreler elenir ve sonuç empty shape olur.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `snapshot` | Sabitin doğrudan shape snapshot'ı | Sonsuz box ve bounds |
| `boxes` | Sabitin `forAllBoxes` taraması | Callback uçları ve sırası |
| `not-same-self` | Aynı sabitin `NOT_SAME` birleşimi | Empty shape snapshot'ı |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "infinity"` ve `view` alanlarını kullanır. `view`, snapshot, boxes veya not-same-self sorgusunu seçer.

## Java yakalama

Math referans kimliği `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` oldu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/shape.rs`, Java `Shapes.INFINITY` sabitinin davranışını `Shapes::infinity` ile uygular; shape sonsuz endpoint'lerle mevcut box üretim yolundan oluşturulur. `examples/behavior_adapter.rs`, snapshot, callback ve self-`NOT_SAME` sorgularını aynı Rust API'sine bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `infinity` senaryosunda sonsuz shape snapshot'ı, callback uçları ve self-`NOT_SAME` empty sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik sabit shape senaryosuyla sınırlıdır. Infinity shape'in world collision akışları sonraki kapsamdır; `Shapes.EPSILON`/`BIG_EPSILON` ve ham `Shapes.create(double...)` yolları ayrı kanıtlarda doğrulanmıştır.
