# P09.c — Entity step-up aday yükseklikleri — [client-parity] birleşik kayıt

Bu kayıt, Java istemcisinin step-up sırasında collider shape'lerinin Y koordinat
düzlemlerinden aday yükseklik üretme yardımcı metodunu Rust'a taşır. Kanıt sınıfı
`client-parity`'dir: aynı hareketli AABB ve world-coordinate collider listesi,
Java 26.2 istemci JAR'ında ve Rust adapter'ında çalıştırılıp karşılaştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/entity/Entity.java:1142-1193`, normal hareket çarpışmasından sonra
  step-up koşulları sağlanırsa collider listesini private
  `collectCandidateStepUpHeights` metoduna verir.
- Aynı metot her collider'ın `VoxelShape.getCoords(Direction.Axis.Y)` sonucunu
  collider sırasıyla tarar; koordinat düzlemleri shape'in yalnız dış
  sınırlarını değil, discrete voxel grid'in ara düzlemlerini de içerebilir.
- `world/phys/shapes/VoxelShape.java:67-71`, Y koordinat düzlemlerini shape'in
  public sorgu sınırından sunar. Java `FloatArraySet` raw float bitlerini
  ayırt eder; `FloatArrays.unstableSort` adayları Java `Float.compare`
  sırasına dizer.

## Sözleşme

- Collider listesi ve her shape'in Y koordinat listesi kaynak sırasıyla taranır.
- Her koordinat için `(coord - boundingBox.minY)` hesaplanır ve sonuç Java
  f32'ye çevrilir.
- `relativeCoord < 0.0F` olan değerler atılır; Java koşulunun `!(...)` biçimi
  korunur. `stepHeightToSkip` ile f32 eşit olan değerler de atılır.
- Bir shape içindeki aday `maxStepHeight` değerini aşınca o shape'in kalan
  koordinatları taranmaz. Sonraki collider'lar taranmaya devam eder.
- Adaylar raw f32 bitlerine göre tekilleştirilir ve Java'nın sayısal float
  sırasına göre sırasız sıralanır. Sonuçta yalnız f32 bit dizisi döner.
- Collider toplama, block-state registry/context çözümü, step-up sonucunun
  yatay hareketle karşılaştırılması, pose/dimensions, on-ground ve collision
  flag kararları bu yardımcı metodun dışındadır.

## Test girdileri ve Java yakalama

Beş özgün sentetik girdi `tools/behavior/cases.py` içindeki
`shape/step-up-candidates/*` kayıtlarında tutulur. Java `MathOracle`, pinned
JAR'daki private metodu reflection ile çağırır. Shape'ler `Shapes.box` üzerinden
oluşturulduğu için ara voxel grid koordinatlarının ve shape sırasının çıktıya
etkisi doğrudan ölçülür.

| Senaryo | Amaç | Java adayları (raw f32 bitleri) |
| --- | --- | --- |
| `empty` | Boş collider yolu | `[]` |
| `sorted-unique` | Ara düzlemler, duplicate ve sıralama | `00000000`, `3e800000`, `3f000000`, `3f400000` |
| `negative-skip` | Negatif değer, exact skip ve max filtresi | `3e000000`, `3ec00000`, `3f000000` |
| `zero-skipped-and-max-break` | Sıfır skip'i ve shape içi erken kesme | `3e800000`, `3f000000` |
| `signed-zero-sort` | Raw signed-zero ayrımı ve Java float sırası | `80000000`, `00000000`, `40000000` |

Java referans kimliği:
`b36a38127171bf6ad8c2d7349fb4b16a70183476fdd2167546f11f7ffd31f6f7`.
Capture, Java'nın aynı girdilerle iki ardışık çalıştırmasında kararlı kaldı.

## Rust doğrulaması

`src/world.rs` içindeki `collect_candidate_step_up_heights`, Java'nın
collider/coordinate traversal sırasını, f32 dönüşümünü, negatif ve skip
filtrelerini, shape başına max break davranışını, raw bit deduplikasyonunu ve
float sıralamasını uygular. `shape_result`, açık collider listesini production
fonksiyonuna taşır. World modül testi sıralı duplicate'ları, negatif/skip
filtrelerini ve erken kesmeyi kontrol eder.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Math corpus'u 2907 kayıtla doğrulandı. Java ve Rust sonuçları **2907/2907**
eşleşti; son doğrulamadaki adapter SHA-256 değeri
`f019f82ed8dc0c43a5891b88fa3e592d95a1c0d30061e681edd81642fb0e1178`.

## Açık kapsam

Java `Entity.collide` içindeki collider toplama ve step-up adaylarının seçilen
hareketle karşılaştırılması, gerçek block-state shape registry/context'i,
entity/world-border collider'ları, pose/dimensions, on-ground/collision flag
state'i ve collision-aware hareket entegrasyonu P09'un açık dallarıdır. Bu
kayıt canlı sunucu kabulü değil, Java istemcisiyle aynı sentetik girdinin
karşılaştırılmasıdır.
