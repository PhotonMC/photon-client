# P09.m — ClientWorld indexed entity collision — [client-parity] birleşik kayıt

Bu kayıt, yüklü entity state'inin `ClientWorld` collision sorgusuna
bağlanmasını taşır. Kanıt sınıfı `client-parity`'dir: entity ID'si, vanilla
entity type ID'si, canlı konumu ve test AABB'si Java'nın gerçek transient
entity section manager sorgusunda ve Rust `ClientWorld` adapter'ında
karşılaştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `client/multiplayer/ClientLevel.java:146-147,522-549,1070-1072`, client level
  entity'leri `TransientEntitySectionManager` ile saklar; add/remove/get/count
  çağrılarını bu depoya yönlendirir ve `getEntities` sorgusunu depodan alır.
- `world/level/entity/TransientEntitySectionManager.java:13-86`, entity
  ekleme, ID lookup ve section storage katmanlarını `EntityLookup` ile bir
  `LevelEntityGetter` görünümünde birleştirir.
- `world/level/entity/EntityLookup.java`, `EntitySectionStorage.java` ve
  `EntitySection.java`, entity'nin block/chunk konumuna göre section'a
  yerleştirilmesini ve AABB sorgusunda seçilmesini yürütür.
- `world/level/EntityGetter.java:56-75`, section getter'dan dönen entity'leri
  source, spectator, collidable ve aynı araç filtrelerinden geçirerek collision
  shape'lerine dönüştürür.
- `world/entity/Entity.java:2372-2378` base entity için `canBeCollidedWith`
  değerini false verir; `world/entity/vehicle/boat/AbstractBoat.java:118-120`
  boat ailesini collider olarak açar.
- `world/entity/EntityType.java:285-291,340-341`, registry entity type'ından
  dimensions alınmasına ve feet-origin bounding box üretilmesine kaynak sağlar.

## Sözleşme

- Her input için yeni bir transient entity section manager oluşturulur; her
  entity ID'si, type ID'si, konumu ve dimensions kutusu manager'a eklenir.
- `entity_count`, section manager'ın gerçek count sonucudur. Collision query,
  düz entity listesini taramak yerine manager'ın `LevelEntityGetter` AABB
  sorgusundan gelir.
- Source ID manager'da bulunuyorsa source entity query dışında bırakılır;
  bulunmuyorsa Java null source davranışı korunur. Query alanı P09.h'deki
  `1.0E-7` broad-phase ve strict kesişim kurallarını korur.
- Rust `ClientWorld` entity map'i aynı ID/type/konum state'ini `apply_entity`
  ile alır. Bu dilimde vanilla boat ailesi type tablosu dimensions'tan
  `1.375 x 0.5625` collider üretir; Java base davranışı nedeniyle player gibi
  desteklenmeyen non-collidable türler aday üretmez.
- Block, entity ve world-border sonuçları P09.l'deki Java sırasıyla tek
  `no_collision_for_entity` sorgusunda birleşir.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik kayıt beş gözlem taşır:

- boş entity index'i;
- query ile kesişen bir boat;
- query dışında kalan bir boat;
- source olarak seçilen boat'ın dışlanması;
- player type'ının collider üretmemesi.

Java oracle, her satırda gerçek `TransientEntitySectionManager` ve
`LevelEntityGetter` section sorgusunu kullanır. Registry'den çözülen type
dimensions'ı fixture bounding box'ına verir; beklenen sonuç Rust çıktısından
türetilmez.

## Rust doğrulaması

`ClientWorld::apply_entity` ile yüklenen entity map'i source ID, canlı konum,
entity count, vehicle root ve desteklenen boat collider state'i için sorguya
bağlandı. Adapter production `ClientWorld` API'sini kullanır; bağımsız aday
listesi fixture'ı kullanılmaz.

Java referans kimliği:
`92263b5264b7aef2519fe3cdc35c23841d8591dcea1b1584748b6d81e220a4f5`.

Math corpus'undaki 2924 kayıt, Java'nın iki ardışık capture çıktısında kararlı
kaldı. Java `verify` başarılı oldu. Rust debug adapter SHA-256 değeri
`4150abbd00889971f3dc4227c4eaaa97de089f7d0fd3e0438ef789e6c0ac5249`, release
adapter SHA-256 değeri
`3c305b97eaab3a6541f396178d54384c61f1d4c54288b56821e26afda3588604` ile iki
karşılaştırma da **2924/2924** eşleşti.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

## Açık kapsam

Entity map'i ve canlı konum state'i birleşik collision sorgusuna bağlandı.
Tam vanilla `EntityType` registry/context çözümü, boat dışındaki özel
collidable türler, spectator/game-mode state'i, kalıcı spatial section index
güncellemeleri, dimensions refresh/scale, block-state registry/context ve
çözülen collision hareketi sonraki P09/P10 dilimleridir. Bu kayıt canlı sunucu
kabulü değildir; vanilla Java istemcisiyle sentetik section/entity state
karşılaştırmasıdır.
