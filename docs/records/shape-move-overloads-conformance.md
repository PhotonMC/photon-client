# P01.d — VoxelShape move overload'ları referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **3 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `VoxelShape.move(Vec3)` ve `VoxelShape.move(Vec3i)` çağrılarının shape koordinatlarını scalar move yoluna taşımasıdır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/VoxelShape.java:73-95`, `world/phys/Vec3.java`, `core/Vec3i.java:61-75` ve `world/phys/shapes/Shapes.java:44-58` incelendi. `MathOracle`, her adımda gerçek Java overload'ını çağırır; beklenen koordinatları test içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- `move(Vec3)` delta nesnesinin üç double alanını sırasıyla X, Y ve Z scalar delta'sı olarak iletir.
- `move(Vec3i)` delta nesnesinin üç int getter sonucunu double'a yükseltip aynı scalar move yoluna iletir. i32 aralığındaki değerler bu dönüşümde tam korunur.
- Empty shape iki overload'da da yeni offset listeleri üretmeden `Shapes.empty()` sonucuna döner.
- Dolu shape'in discrete hücre işgali değişmez; yalnızca X, Y ve Z koordinat düzlemlerine kendi delta'ları eklenir.
- Koordinat güncellemeleri Java double toplama sırasını ve signed zero/özel değer bitlerini korur. Sonuçlar toleranssız ham f64 bitleriyle karşılaştırılır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Vec3 | 1 | Asimetrik box ve negatif/pozitif double delta |
| Vec3i | 1 | Negatif koordinatlı box ve üç integer block delta'sı |
| empty | 1 | Empty shape üzerinde Vec3 ve Vec3i overload zinciri |

Girdiler `family: "math"`, `operation: "shape"`, `kind: "single"` ve `steps` alanlarını kullanır. Scalar adım `op: "move"`, Vec3 overload'u `op: "move_vec3"` ve Vec3i overload'u `op: "move_block_pos"` taşır. Vec3 delta'sı `delta` altında f64 bit string'leri, Vec3i delta'sı aynı alan altında ondalık i32 string'leri taşır. Her adımın sonucu Java shape snapshot'ı olarak kaydedilir.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Bu dilimin bulunduğu güncel math referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğindedir. Referans iki JVM çalıştırmasıyla kararlı bulundu. Güncel birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle doğrulandı.

Math debug/release raporları `compare-00tgiqc4` ve `compare-aylr4ozm`; birleşik debug/release raporları `compare-5bjkz31s` ve `compare-zfkba6ts` dizinlerindedir, `.data/runs/behavior/` altında bulunur. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/photon-core/src/shape.rs`, Java `VoxelShape.move(Vec3)` ve `move(Vec3i)` çağrılarını `VoxelShape::move_by_vec3` ve `VoxelShape::move_by_block_pos` ile uygular; iki yöntem de mevcut scalar `move_by` yoluna bağlanır. `examples/behavior_adapter.rs` `move_vec3` ve `move_block_pos` shape adımlarını bu API'ye bağlar. Davranış sınırları [referans sözleşmesinde](#sözleşme) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Üç yeni senaryoda Vec3 double delta'sı, Vec3i integer delta'sı ve empty shape üzerinde iki overload zinciri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik `kind: "single"` shape senaryosuyla ve Vec3/Vec3i move adımlarıyla sınırlıdır. `optimize`, `forAllEdges`, doğrudan koordinat listeleri ve istemci hareketindeki shape taşıma zinciri sonraki kapsamdır. Scalar `move(double,double,double)` davranışı mevcut [temel shape kanıtında](shape-conformance.md) yer alır.
