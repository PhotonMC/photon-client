# P09.a — ClientWorld blok çarpışma şekli toplama — [client-parity] birleşik kayıt

Bu kayıt, yüklenmiş `ClientWorld` block-state ID'lerinden çarpışma şekillerini
toplayan dar dilimi Java 26.2 ile karşılaştırır. Kanıt sınıfı
`client-parity`'dir: aynı sentetik blok yerleşimleri ve sorgu AABB'si Java
istemci JAR'ında ve Rust adapter'ında çalıştırılmıştır.

## Java kaynak incelemesi

İncelenen Java yolları şunlardır:

- `world/level/CollisionGetter.java:96-107`, normal blok çarpışması sorgusunu
  boş `CollisionContext` ile `BlockCollisions` iterator'ına yönlendirir.
- `world/level/BlockCollisions.java:18-112`, sorgu sınırlarını `1.0E-7`
  epsilon ve bir blokluk payla genişletir; chunk'ı x/z'den seçer, block-state
  collision shape'ini world konumuna taşır, tam blok için AABB kesişimini ve
  diğer shape'ler için `BooleanOp.AND` kesişimini test eder.
- `core/Cursor3D.java:3-69`, hücreleri x hızlı, sonra y, sonra z sırasıyla
  gezer; üç yüzeye aynı anda denk gelen köşe hücrelerini `TYPE_CORNER` olarak
  sınıflandırır.
- `world/level/BlockGetter.java:24-34` ve
  `world/level/block/state/BlockBehaviour.java:686-692`, block-state ve
  collision-shape çağrı sınırlarını tanımlar.
- `world/level/block/SlabBlock.java:31-64`, alt/üst slab shape'lerinin yarım
  yüksekliklerini ve çift slab için tam blok shape'ini seçer.

## Sözleşme

- Sorgu AABB'si Java `BlockCollisions` ile aynı epsilon ve bir blok payıyla
  taranır.
- Cursor sırası x, y, z'dir; köşe hücreleri atlanır.
- Yalnızca yüklenmiş chunk'taki block-state ID'si okunur. State ID için shape
  tablosu çağıran tarafından sağlanır; tabloda olmayan ID çarpışma üretmez.
- Tam blok shape'i için ham blok AABB'siyle strict kesişim aranır. Diğer
  shape'ler world koordinatına taşınır ve `Shapes.joinIsNotEmpty(..., AND)` ile
  test edilir; yalnızca temas eden yüzler çarpışma sayılmaz.
- Sonuç shape'i, block position ve world koordinatlarına taşınmış shape ile
  birlikte cursor sırasını korur.
- Block-state registry/context kuralları, gerçek section palette çözümü,
  entity/world-border collider'ları, pose/dimensions, on-ground/flag ve step-up
  kararları bu dilimin dışındadır.

## Test girdileri ve Java yakalama

Dört özgün sentetik girdi `tools/behavior/cases.py` içindeki
`shape/collect-block-collisions/*` kayıtlarında tutulur. Java `MathOracle`,
özgün JAR'daki `BlockCollisions` iterator'ını, sentetik bir `CollisionGetter`
üzerinden doğrudan çağırır. Rust adapter aynı yerleşimleri yüklenmiş
`ClientWorld` içine yazar ve production `collect_block_collision_shapes`
fonksiyonunu çağırır.

| Senaryo | Amaç | Java gözlemi |
| --- | --- | --- |
| `empty` | Boş tarama yolu | Sonuç listesi boş |
| `full-and-slabs` | Tam blok, alt/üst slab, çift slab ve cursor sırası | `(0,0,0)`, `(1,0,0)`, `(2,0,0)`, `(1,1,0)` sırasıyla dört shape |
| `touching-faces` | Strict kesişim ve yüz teması | Yalnız `(0,0,0)` alt slabı döner; komşu yüzlere değen üç tam blok elenir |
| `chunk-boundary` | x=15/16 chunk geçişi | `(15,0,0)` tam bloktan sonra `(16,0,0)` üst slabı döner |

Java referans kimliği:
`d59ad44b6d897661797f09e1e249794e56cf088e052c4b64789660215da616dc`.
Capture, Java'nın aynı girdilerle iki ardışık çalıştırmasında kararlı kaldı.

## Rust doğrulaması

`src/world.rs` içindeki `collect_block_collision_shapes`, yüklenmiş chunk
section'larından block-state ID'sini okur; Java epsilon sınırlarını, cursor
tipi filtresini, tam blok hızlı yolunu, world-coordinate shape taşımasını ve
strict BooleanOp kesişimini uygular. `shape_result`, shape tablosunu ve
sentetik section'ları production `ClientWorld` API'sine bağlar. World modül
testi cursor sırasını, yarım shape taşımasını, eksik/empty shape filtrelerini
ve yüz temasını kontrol eder.

```text
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Math corpus'u 2902 kayıtla doğrulandı. Java ve Rust sonuçları **2902/2902**
eşleşti; bu dilimin dört yeni `collect_block_collisions` girdisi de buna
dahildir.

## Açık kapsam

Gerçek block-state registry'sinden shape çözümü, palette/state property
bağlantısı, chunk section'ın canlı ağ akışından bu API'ye taşınması,
entity/world-border shape'leri, step-up, pose/dimensions, on-ground ve hareket
entegrasyonu açık kalır. Bu kayıt canlı sunucu kabulü değil, Java istemcisiyle
aynı sentetik girdinin karşılaştırılmasıdır.
