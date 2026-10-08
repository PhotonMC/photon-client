# P01.d — Shapes.create(AABB) sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `Shapes.create(AABB)` overload'ının AABB endpoint'lerini scalar create yoluna aktarmasını, AABB normalizasyonunu ve küçük aralık eşiğini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:52-99` ve `world/phys/AABB.java:22-29` incelendi. `MathOracle`, `Shapes.create(AABB)` çağrısını doğrudan yapar; beklenen shape listesi yeniden hesaplanmaz.

## Davranış sözleşmesi

- `Shapes.create(AABB)` AABB'nin altı normalize edilmiş endpoint'ini `create(double...)` yoluna taşır.
- AABB constructor'ı her eksende `Math.min`/`Math.max` ile normalizasyon yaptığı için ters verilen giriş endpoint'leri create çağrısına sıralı ulaşır.
- Herhangi bir eksen genişliği `1e-7` değerinden küçükse sonuç empty shape olur. Diğer girdiler scalar create yolunun coordinate grid ve box üretim davranışını korur.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `reversed` | Her eksende ters endpoint'li AABB | AABB normalizasyonundan sonra oluşan shape snapshot'ı |
| `empty` | Bir ekseni `1e-7` altı olan AABB | Empty shape snapshot'ı |
| `array` | Birim küp dışına taşan ve grid'e oturmayan endpoint'ler | Array shape'in box ve bounds snapshot'ı |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "create_aabb"` ve `aabb` alanlarını kullanır. Sonuç scalar shape snapshot formatındaki `empty`, `boxes` ve non-empty durumda `bounds` alanlarını taşır.

## Java yakalama

Math referans kimliği `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` oldu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/photon-core/src/shape.rs`, Java `Shapes.create(AABB)` overload'ını `Shapes::create_from_aabb` ile uygular; AABB'nin normalize edilmiş endpoint'lerini mevcut scalar box üretim yoluna bağlar. `examples/behavior_adapter.rs`, `kind: "create_aabb"` girdisini bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `create_aabb` senaryosunda ters endpoint normalizasyonu, küçük aralık empty sonucu ve birim küp dışı coordinate grid sonucu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik AABB girdisiyle sınırlıdır. `Shapes.create(double...)` için ham koordinat yolları ayrı [kanıtta](shape-create-coords-conformance.md), `Shapes.INFINITY` sabiti ayrı [kanıtta](shape-infinity-conformance.md) doğrulanır; shape üretimindeki diğer package-private merger yardımcıları sonraki kapsamdır.
