# T25c — ClientWorld ItemEntity tick (düşme, sekme, kayma) — [client-parity]

Bu kayıt, sunucunun yerleştirdiği bir `ItemEntity`'nin istemci level tikinde
Java 26.2 ile Rust world kopyasında aynı konum, hareket ve `onGround`
değerlerini üretmesini kapsar. Sunucu eşyanın konumunu seyrek gönderir; paketler
arasında eşyayı istemci kendi tikiyle ilerletir. Bu yüzden kırılan bloktan düşen
eşya düşerken görünür. Kanıt sınıfı `client-parity`'dir: aynı özgün sahne
Java'nın gerçek `ClientLevel`'ında (`MovementOracle.buildScene`, gerçek chunk
bölümleri) ve Rust adapter'ında koşturulur; her tikte çıktı bit bit karşılaştırılır.

## Java kaynak incelemesi

- `world/entity/item/ItemEntity.java:118-192`: yerçekimi `0.04`, yerdeyken ve
  yatay hız eşiğin altındayken `(tickCount + id) % 4 != 0` tiklerinde hareket
  atlanır; aksi hâlde `move`, sürtünme (`0.98F`, yerde bloğun sürtünmesiyle
  çarpılır) ve yerdeki aşağı hızın `-0.5` ile sekmesi. `:195-197`: sürtünmeyi
  veren blok `getOnPos(0.999999F)`.
- `world/entity/Entity.java:712-768`: `move` çarpışmayı çözer, hareketi ve
  bayrakları uygular, `onGround` yalnız dikey hareket varken güncellenir, sonunda
  bloğun hız çarpanı yatay hıza uygulanır. `:672-700`, `:1055-1075`: destek
  bloğu ve `getOnPos`. `:1142-1195`: `maxUpStep` 0 olduğundan basamak yok.
- `world/entity/Entity.java:904-960`, `world/level/block/SlimeBlock.java:30-38`:
  yerdeki eşya için `stepOn`; slime yatay hızı `0.4 + |y| * 0.2` ile yavaşlatır.
- `world/entity/Entity.java:355-361`, `network/protocol/game/VecDeltaCodec.java`,
  `client/multiplayer/ClientPacketListener.java:640-650,729-754`: sunucunun
  göreli konum paketleri entity'nin gerçek konumuna değil, `packetPositionCodec`
  tabanına uygulanır. Taban yalnız `AddEntity`'de, `MoveEntity`'nin çözdüğü
  konumda ve `EntityPositionSync`'te güncellenir. İstemci eşyayı kendi
  ilerlettiğinde taban ile konum ayrışır; tabanı tutmayan bir istemci her paketle
  eşyayı yanlış yere atar (canlıda eşyanın sunucu konumuna ışınlanması).
- İstemcide eşya yerel olarak yetkili değildir: `Entity.move`'daki genel sekme
  (`restituteMovementAfterCollisions`) çalışmaz; yalnız `ItemEntity.tick`'in kendi
  sekmesi vardır. `ClientPacketListener.handleMoveEntity` konumu ve `onGround`'u
  doğrudan yazar (`setOnGround` destek bloğunu da yeniler).

## Kapsam ve dal kararı

`ItemEntity.tick` istemci kolu ilk kez ölçülüyor; mevcut kayıtlar yalnız
mermi/blok/küre tiklerini kapsar. Corpus tek `item_entity_ticks` kaydıdır, 32
satır (dördü sunucu paketli: göreli hareket, mutlak konum, hız paketi, dinlenirken konum eşitleme; Java'nın gerçek `handleMoveEntity`/`handleEntityPositionSync`/`handleSetEntityMotion` işleyicileriyle): blok düşüşü ve yüksekten bırakma, dinlenme (kimliğe göre 4 tikte bir
hareket), taş/buz/paketli buz/slime/ruh kumu/bal zeminlerinde kayma ve düşme,
duvar, tavan, basamak, iki zemin arasındaki kenar (destek bloğu), yarım blok,
çit, halı ve boşluğa düşme. Zemin türleri aynı kodun yalnız sabit tablosunu
değiştirdiği için satırdır, ayrı dilim değildir.

## Akışkan (T25d)

`Entity.baseTick`'in `updateFluidInteraction` taraması tikin başında, ItemEntity
tikinin sonunda bir kez daha çalışır: kutunun değdiği su ve lav, akışkanın
ayaktan yüksekliği ve akıntı. Akıntı oyuncu olmayan entity'ye **ortalaması
değil yönü** (`normalize`) olarak uygulanır (`Tracker.applyCurrentTo`; oyuncuda
ortalama). Yükseklik 0.1'i aşarsa eşya `0.99` (su) ya da `0.95` (lav) ile
yavaşlar ve dikey hızı 0.06'nın altındaysa `+5e-4` ile yükselir; yoksa yerçekimi.
Tarama oyuncuyla ortak koddadır (`voxel-world/fluid.rs`). `item_entity_ticks`'e
19 satır eklendi: havuza düşme (düz ve yanlamasına), derin suda yükselme, sığ su,
yukarı fırlatma, buz/slime üzerinde su, 1/9 ve 2/9 yüksekliğinde yüzey (0.1
eşiği), akan su (dere, uç, şelale), lav (havuz, sığ, akış), su ile lav arası ve
su altında duvar.

## Sapmalar (açık)

- Blok içi etkiler (`checkInsideBlocks`: ağ, böğürtlen, toz kar, kabarcık sütunu,
  ateş) ve `stepOn`'un slime dışındaki dalları (harekete etkisiz) taşınmadı.
- `FAST_LAVA` (Nether) ortamının lav akıntı ölçeği (0.007) yok; hep 0.0023333.
- Çarpışmada dünya sınırı ve çarpışan entity'ler (tekne, shulker) yok;
  iskele gibi entity'ye bağlı şekiller için eşya o tikte konumunu korur.
- `pickupDelay`, birleşme ve yaş sınırı çizimi etkilemediği için yok.
