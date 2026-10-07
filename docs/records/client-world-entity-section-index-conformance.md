# P09.r — ClientWorld kalıcı entity section index güncellemeleri — [client-parity] birleşik kayıt

Bu kayıt, `ClientWorld` içindeki entity'lerin ekleme, hareket ve silme sonrasında
Java'daki transient section üyeliğiyle aynı sorgulanabilir durumu taşımasını
kanıtlar. Kanıt sınıfı `client-parity`'dir; canlı sunucu kabulü değildir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `world/level/entity/TransientEntitySectionManager.java:13-139`, entity'yi
  `SectionPos.asLong(entity.blockPosition())` ile section'a ekler; `onMove`
  eski section'dan çıkarıp yeni section'a ekler ve boş section'ı siler;
  `onRemove` section üyeliğini, entity lookup kaydını ve boş section'ı temizler.
- `world/level/entity/EntitySectionStorage.java:31-90`, AABB sorgusunda yatay
  aralığı `-2/+2`, dikey aralığı `-4/+0` section payıyla tarar, sonra erişilebilir
  ve boş olmayan section'larda kesin AABB kesişimini uygular.
- `world/level/entity/EntitySection.java`, add/remove ve kesin
  `entity.getBoundingBox().intersects(bb)` kontrolünü taşır.
- `world/level/entity/LevelEntityGetterAdapter.java`, AABB sorgusunu section
  storage'a; ID, count ve tüm entity sorgularını lookup'a yönlendirir.
- `world/level/SectionPos.java:77-87` section koordinatını önce floor edilmiş
  block koordinatının `>> 4` sonucu olarak üretir.
- `world/entity/Entity.java:3811-3826,3937-3949`, konum değişiminde
  `levelCallback.onMove()` ve silinmede `onRemove()` callback'lerini çağırır.

## Davranış sözleşmesi

- Her ekli entity, feet konumunun floor/block koordinatından türetilen tek bir
  `(section_x, section_y, section_z)` üyeliğine sahip olur.
- Aynı section içindeki hareket yeni bir section oluşturmaz. Section sınırı
  geçilince eski üyelik kaldırılır, yeni üyelik eklenir ve boş eski section
  silinir.
- AABB entity lookup'ı Java'nın yatay `-2/+2` ve dikey `-4/+0` genişletilmiş
  section aralığını kullanır; adaylar kesin bounding-box kesişmesinden sonra
  collision filtresine girer.
- Entity silindiğinde hem entity lookup hem section index güncellenir; silinmiş
  section veya eski konum artık collision adayı üretmez.
- Bu state, `ClientWorld` collision sorgusunun mevcut block, entity ve
  world-border birleşimine bağlanır. Section index yalnız wire tüketimini değil,
  sorgulanabilir istemci dünya durumunu ilerletir.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`world/client-collision/entity-section-index-updates` kaydı iki iz içerir:
aynı section'da hareket, yatay ve dikey section sınırı geçişi, silme ve boş
section temizliği; ayrıca komşu section'ın diğer entity silindikten sonra
korunması.

Java oracle gerçek `TransientEntitySectionManager<Entity>` ve AABB getter'ını
kullanır. Hareket olayı gerçek `setPosRaw` callback yolundan geçirilir; remove
olayı `Entity.setRemoved` ile uygulanır. Her query gözlemi entity sayısını,
section sayısını, `noEntityCollision` ve birleşik `noCollision` sonucunu üretir.
Beklenen çıktı Rust adaptöründen türetilmemiştir.

Java referans kimliği:
`a165d869e31af0486e57db621508d89a8a0e7ae474074026cc21cec3a9743a9c`.

Math corpus'u iki ardışık Java capture çıktısında kararlı kaldı; bu kayıt
eklendiğinde toplam 2929 kayıt karşılaştırıldı. Rust debug ve release adapter'ları
her ikisinde de **2929/2929** eşleşti, fark `null` kaldı.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

Rust tarafında `ClientWorld` section üyelik tablosu entity add/remove, packet
hareketi, tick, passenger konumlandırması ve sentetik test hareket API'siyle
güncellenir. AABB sorgusu Java section aralıklarını tarayıp mevcut entity type,
dimensions ve collision filtrelerine bağlanır.

## Açık kapsam

Tam EntityType/context registry'si, Minecart ve diğer özel source dispatch'leri,
yaşayan entity'nin tüm `isPushable` koşulları, çözülen movement AABB'i, komşu
stairs güncellemesi, `refreshDimensions` yan etkileri, gerçek flags/abilities ve
genel entity tick/lifecycle dalları sonraki P09/P08 dilimleridir. Bunlar bu
section index kanıtının yerine geçmez.
