# P01.d — VoxelShape min/max sütun sorguları referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `VoxelShape.min(axis, b, c)` ve `VoxelShape.max(axis, b, c)` overload'larının verilen iki koordinatla seçilen sütundaki dolu hücreleri taramasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:141-161`, `world/phys/shapes/DiscreteVoxelShape.java:98-136` ve `core/AxisCycle.java:1-78` incelendi. `MathOracle`, shape'i Java `Shapes.or(first, tail...)` yoluyla kurar ve her sorguyu doğrudan `min` veya `max` overload'una verir; beklenen koordinatları test içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- `axis` sonuç koordinatının eksenidir. `b` ve `c`, bu eksene dik iki eksende sütun seçen koordinatlardır.
- Java `AxisCycle` sırası sütun eksenlerini şöyle seçer:

  | Sonuç ekseni | `b` ekseni | `c` ekseni |
  | --- | --- | --- |
  | X | Y | Z |
  | Y | Z | X |
  | Z | X | Y |

- Her koordinat, mevcut voxel aralığını Java'nın binary search sınırlarıyla seçer. İlk düzlemin altındaki veya son düzlemdeki koordinat geçerli bir hücre sütunu seçmez.
- `min(axis, b, c)`, seçilen sütunda baştan sona taranan ilk dolu hücrenin `axis` başlangıç düzlemini döndürür.
- `max(axis, b, c)`, seçilen sütunda sondan başa taranan son dolu hücrenin `axis` bitiş düzlemini döndürür.
- Sütun koordinatlardan herhangi biri dışındaysa veya sütunda dolu hücre yoksa `min` pozitif sonsuz, `max` negatif sonsuz döndürür.
- Sonuçlar IEEE-754 f64 ham bit string'leriyle karşılaştırılır; tolerans veya NaN normalizasyonu uygulanmaz.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| empty | 1 | Java'da empty olan epsilon altı sliver ve geçerli görünen iki sütun sorgusu |
| disjoint | 1 | Ayrık iki box, üç sonuç ekseni için sütun sorguları ve dış sütun sorgusu |
| negative | 1 | Negatif ve pozitif koordinatları birlikte taşıyan iki box ve üç eksen sorgusu |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "axis_limits_at"`, `shapes` ve `queries` alanlarını kullanır. Her query `op`, `axis`, `b` ve `c` taşır; `b` ile `c` f64 bit string'leridir. Sonuç, sorgu sırasını koruyan `values` dizisinde ham f64 bit string'leri taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Bu dilimin bulunduğu güncel math referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı.

Math debug/release raporları `compare-00tgiqc4` ve `compare-aylr4ozm`; birleşik debug/release raporları `compare-5bjkz31s` ve `compare-zfkba6ts` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/shape.rs`, Java `VoxelShape.min(axis, b, c)` ve `max(axis, b, c)` overload'larını `VoxelShape::min_at` ve `VoxelShape::max_at` ile uygular. `column_axes`, `find_index` ve `is_full_wide` yardımcıları Java'nın `AxisCycle` sütun seçimini, hücre aralığını ve dolu hücre taramasını korur. `examples/behavior_adapter.rs` `axis_limits_at` JSONL yolunu bu API'ye bağlar. Davranış sınırları [referans sözleşmesinde](#sözleşme) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Üç yeni senaryoda empty shape, ayrık iki box'ın X/Y/Z sütunları, negatif koordinatlar ve dış sütun için `+∞/−∞` sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik iki-box shape listesi ve seçilmiş sütun sorgularıyla sınırlıdır. Koordinat listelerinin doğrudan dışa açılması, world collider toplama ve bu sorguların istemci hareketindeki çağrı zinciri sonraki kapsamdır. Global `min(axis)` ve `max(axis)` sorguları [ayrı kanıtta](shape-axis-limits-conformance.md) doğrulanır.
