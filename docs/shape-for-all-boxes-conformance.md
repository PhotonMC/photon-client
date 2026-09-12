# P01.d — VoxelShape forAllBoxes sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `VoxelShape.forAllBoxes` callback'inin empty shape, tek box ve birden fazla dolu bölge için Java merge sırasını ve ham f64 uçlarını taşımasını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:117-139`, `DiscreteVoxelShape.java:220-222` ve `BitSetDiscreteVoxelShape.java:160` içindeki `forAllBoxes` merge taraması incelendi. `MathOracle`, callback'i doğrudan çalıştırır; beklenen box listesi yeniden hesaplanmaz.

## Davranış sözleşmesi

- `forAllBoxes(consumer)` shape'in dolu hücrelerini Java'nın `mergeNeighbors: true` yoluyla tarar.
- Her callback çağrısı coordinate grid'inden elde edilen `(minX,minY,minZ,maxX,maxY,maxZ)` uçlarını taşır; callback sırası gözlenebilir davranışın parçasıdır.
- Empty shape callback'i hiç çağırmaz. Komşu dolu hücreler önce Z şeritlerinde, sonra X ve Y boyunca birleştirilir.
- `toAabbs()` aynı callback yolunu liste oluşturarak tüketir; iki API aynı box sırasını paylaşır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `empty` | `1e-7` altı sliver | Boş callback sonucu |
| `box` | Asimetrik tek kutu | Tek callback ve altı ham f64 uç |
| `join` | Üç ayrık 3B kutunun OR birleşimi | Java merge sırasındaki tüm callback uçları |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "boxes"` ve `shapes` alanlarını kullanır. Sonuç `boxes` dizisinde her callback uçlarını altı ham f64 bit string'iyle taşır.

## Java yakalama

Math referans kimliği `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` oldu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/shape.rs`, Java'nın birleşik box taramasını `VoxelShape::for_all_boxes` callback'iyle uygular; `to_aabbs` aynı callback'i kullanır. `examples/behavior_adapter.rs`, callback uçlarını JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `boxes` senaryosunda callback çağrı sırası ve altı uç değeri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik box senaryosuyla sınırlıdır. İç discrete `forAllBoxes(..., false)` overload'ı, face taraması ve world collider toplama sonraki kapsamdır.
