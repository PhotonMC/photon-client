# P01.d — VoxelShape optimize ve getCoords referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `VoxelShape.optimize()` ile shape'in Java kutu iterasyonu üzerinden yeniden kurulmasını ve `getCoords` coordinate plane sonuçlarını içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:71,96-100,117-139`, `DiscreteVoxelShape.java:220-221`, `BitSetDiscreteVoxelShape.java:160-208` ve `Shapes.java:139-172` incelendi. `MathOracle`, optimize öncesi ve sonrası `getCoords` listelerini doğrudan Java shape nesnesinden okur.

## Davranış sözleşmesi

- `getCoords(axis)` eksenin artan coordinate plane listesini döndürür; empty shape'in her eksende tek `0.0` düzlemi vardır.
- `optimize()` `forAllBoxes` çağrısının Java merge sırasındaki her kutusunu `Shapes.box` ile yeniden üretir ve bunları `Shapes.joinUnoptimized(..., OR)` yoluyla soldan sağa birleştirir.
- Optimize sonucu dolu hücre işgalini korur; komşu kutuların gereksiz ara coordinate plane'leri kaldırılır, ayrık kutuların geometriyi tanımlayan düzlemleri korunur.
- Empty shape optimize edildiğinde yeni kutu üretmeden empty shape döner.
- Coordinate değerleri Java double sonuçları olarak, karşılaştırma ham f64 bitleriyle yapılır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| empty | 1 | `1e-7` altı aralığın optimize edilmesi ve üç eksenin koordinat listeleri |
| temas eden satır | 1 | X ekseninde üç komşu kutu; optimize öncesi beş, sonrası iki X düzlemi |
| ayrık | 1 | Üç farklı 3B kutu; optimize sonrası gerekli birleşik düzlemler |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "optimize"` ve `shapes` alanlarını kullanır. Java oracle, girdileri önce `Shapes.joinUnoptimized(..., OR)` ile kurar; `events` ilk shape ve `shape.optimize()` snapshot'larını taşır. Her snapshot `empty`, `boxes`, `bounds` ve X/Y/Z `coords` listelerini içerir.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Bu dilimin bulunduğu güncel math referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı.

Math debug/release raporları `compare-00tgiqc4` ve `compare-aylr4ozm`; birleşik debug/release raporları `compare-5bjkz31s` ve `compare-zfkba6ts` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/shape.rs`, Java `VoxelShape.optimize()` yolunu `VoxelShape::optimize` ile; public `getCoords` gözlemini de `VoxelShape::coords` ile uygular. Optimize, mevcut `to_aabbs` merge sırasındaki kutuları yeniden kurup OR ile birleştirir. Davranış sınırları [referans sözleşmesinde](#sözleşme) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Üç yeni senaryoda empty sonuç, temas eden kutularda coordinate plane sadeleşmesi ve ayrık kutularda düzlem korunumu karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "optimize"` shape senaryosuyla ve kutu/coordinate plane snapshot'larıyla sınırlıdır. [forAllEdges](shape-edges-conformance.md), [forAllBoxes](shape-for-all-boxes-conformance.md) ve [joinUnoptimized](shape-join-unoptimized-conformance.md), doğrudan üretim shape'lerinin tüm index merger türleri ve world collider toplama sonraki kapsamdır.
