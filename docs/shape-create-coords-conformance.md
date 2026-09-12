# P01.d — Shapes.create(double...) sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **3 yeni Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `Shapes.create(double...)` overload'ının ham koordinat uç noktalarıyla shape üretme yollarını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:52-99`, `BitSetDiscreteVoxelShape.java:23-42` ve `ArrayVoxelShape.java:9-45` incelendi. `MathOracle`, skaler overload'ı doğrudan çağırır; beklenen shape listesi yeniden hesaplanmaz.

## Davranış sözleşmesi

- Herhangi bir eksen için `max - min < 1.0E-7` olduğunda sonuç `Shapes.empty()` olur. Ters endpoint'ler bu kontrol nedeniyle ayrıca hata üretmeden empty sonuç verir.
- Üç eksenin endpoint'leri `findBits` içinde `0` ile `3` arasındaki grid aralıklarına oturuyorsa dolu hücreler `CubeVoxelShape` olarak kurulur; hücre sınırları Java `Math.round` ile seçilir.
- Endpoint'lerden biri birim küp/grid sınırlarına sığmazsa `ArrayVoxelShape` döner ve üç eksenin ham `[min, max]` koordinatlarını korur.
- Overload, min/max sırası için `box` gibi bir doğrulama yapmaz. Doğrulama isteyen çağrı `Shapes.box` yoludur.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `cube` | Üç eksende 0–1 grid'ine oturan ham endpoint'ler | Cube shape snapshot'ı |
| `reversed` | Ters sıralı ham endpoint'ler | Empty shape snapshot'ı |
| `array` | Birim küp/grid sınırları dışındaki endpoint'ler | Array shape'in box ve bounds snapshot'ı |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "create_coords"` ve `box` alanlarını kullanır. Sonuç `empty`, `boxes` ve non-empty durumda `bounds` alanlarını taşır; endpoint'ler f64 bit string'leridir.

## Java yakalama

Math referans kimliği `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` oldu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/shape.rs`, Java `Shapes.create(double...)` overload'ını `Shapes::create_from_coords` ile uygular. `examples/behavior_adapter.rs`, `kind: "create_coords"` girdisini ham f64 endpoint'leriyle bu API'ye taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Üç yeni `create_coords` senaryosunda cube, ters endpoint empty sonucu ve ArrayVoxelShape fallback'i karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik ham koordinat girdisiyle sınırlıdır. `findBits` yardımcı metodunun package-private doğrudan erişimi, özel NaN payload'ları ve shape üretimindeki diğer merger dalları ayrı kapsamdır.
