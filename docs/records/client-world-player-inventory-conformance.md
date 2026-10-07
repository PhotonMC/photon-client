# P08.m — [server-conformance] LocalPlayer inventory/container state'i

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte
tutar. Kapsam, vanilla clientbound inventory/container paketlerinin local
player `ClientWorld` kopyasında sorgulanabilir ham slot ve carried state'ine
bağlanmasıdır. Canlı kanıt sınıfı `server-conformance`'dır: Rust probe'u
Mojang vanilla 26.2 dedicated server'ına bağlanır ve server'ın gönderdiği
packet state'ini Rust world kopyasıyla birlikte ölçer. Aynı girdi Java
istemcisiyle karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundContainerSetContentPacket.java:11-22`, container ID ve state
ID'den sonra optional item stack listesini ve carried item'i taşır.
`ClientboundContainerSetSlotPacket.java:9-53`, container ID, state ID, signed
short slot ve optional item stack taşır. `ClientboundSetCursorItemPacket.java:9-20`
yalnız optional carried stack taşır. `ClientboundSetPlayerInventoryPacket.java:10-25`
doğrudan player inventory index'i için VarInt ve optional stack taşır.

Bu paketler play dispatch tablosunda `container_set_content` için `18`,
`container_set_slot` için `20`, `set_cursor_item` için `96` ve
`set_player_inventory` için `108` ID'sini kullanır. Optional stack'in dolu
wire biçimi count, item registry ID'si ve component patch ID listelerini;
boş biçimi count `0` değerini taşır.

`ClientPacketListener.java:1400-1455`, container ID `0` için player inventory
menu'sünün content ve slot state'ini günceller. Cursor paketi
`AbstractContainerMenu.setCarried` yoluna gider. Direct inventory paketi
`Player.getInventory().setItem` çağrısını kullanır. `Inventory.java:36-56`
player inventory index'lerini tanımlar; hotbar index'leri menu slotu
`36..44` aralığına, main inventory index'leri `9..35` menu slotlarına karşılık
gelir. `AbstractContainerMenu.java:618-630,769-775` slot/state ID ve carried
güncellemesinin ayrı tutulduğunu gösterir.

## Sözleşme

- `container_set_content` packet'i container ID, state ID, listedeki bütün
  optional stack'ler ve carried stack'i packet sonuna kadar tüketir.
- Local player mevcutken container ID `0` content'i 46 menu slotuna, carried
  item'e ve menu state ID'sine birlikte yazar. Her stack count, item registry
  ID'si ve positive/negative component type ID listeleri ham olarak tutulur.
- `container_set_slot` container ID `0` için belirtilen slotu değiştirir ve
  menu state ID'sini packet state ID'sine ayarlar. Boş optional stack slotta
  `None` olarak sorgulanır.
- `set_cursor_item` carried item'i değiştirir; bu dilimde GUI creative ekranı
  için Java'nın remote-slot özel dalı uygulanmaz.
- `set_player_inventory` packet'i `0..8` inventory index'ini menu slotu
  `36..44` aralığına, `9..35` index'ini aynı numaralı menu slotuna dönüştürür.
  Bu paket menu state ID'sini değiştirmez.
- Local player henüz yoksa veya content container ID'si `0` değilse raw packet
  observation korunur, `applied_to_world=false` olur. Custom menu slot haritası
  bu dilimin dışındadır.
- Yeni local player oluşturulduğunda inventory slot map'i, carried item ve
  menu state ID'si yeni constructor varsayılanlarına döner; sonraki content ve
  slot paketleri state'i yeniden kurar.

## Rust doğrulaması

`ClientItemStackState`, optional olmayan stack'lerin count, item registry ID'si
ve component type ID listelerini taşır. Content parser listedeki bütün
stack'leri `ContainerSetContentObservation.items` alanında, carried stack'i
ayrı state alanında korur; önceki seçili slot özetleri de geriye dönük gözlem
alanları olarak tutulur. Slot, cursor ve direct inventory parser'ları Java
alan sırasını ve trailing payload sınırını doğrular.

`ClientWorld::apply_local_player_container_content` container `0` content'ini
46 slotluk map'e ve state ID'ye yazar. Slot ve cursor handler'ları ilgili local
player state'ini günceller; direct inventory handler'ı Java inventory index
eşlemesini uygular. Her observation `applied_to_world` ile world uygulamasını
kaydeder. Unit test; dolu/boş slotları, carried state'i, state ID değişimini,
direct hotbar dönüşümünü ve respawn temizliğini doğrular.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
başlangıç oyun modu creative, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Rust probe bağlandıktan sonra vanilla server konsolunda her oturum için
oyuncu survival'a alındı ve hotbar slotu component-bearing honey bottle ile
hazırlandı; poison effect de aynı slot üzerinde verildi:

```text
gamemode survival <probe-user>
item replace entity <probe-user> hotbar.0 with minecraft:honey_bottle[minecraft:custom_model_data={floats:[1.0f]}]
effect give <probe-user> minecraft:poison 60 0 true
```

Rust probe:

```text
cargo run --locked --offline --example survival_honey_bottle_poison_cleanup_probe -- 127.0.0.1:25568 <probe-user>
```

İki bağımsız geçerli vanilla oturumunda gözlenen sonuçlar:

| Gözlem | `P08mC` | `P08mD` |
| --- | ---: | ---: |
| login player ID | `4202` | `4334` |
| initial content | `container=0, state=1, items=46, all empty, carried=empty, applied=true` | `container=0, state=1, items=46, all empty, carried=empty, applied=true` |
| preparation slot | `state=2, slot=36, count=1, item=1412, components=[17], applied=true` | `state=2, slot=36, count=1, item=1412, components=[17], applied=true` |
| remainder slot | `state=3, slot=36, count=1, item=1149, components=[], applied=true` | `state=3, slot=36, count=1, item=1149, components=[], applied=true` |
| final inventory state ID | `3` | `3` |
| final slot 36 | `count=1, item=1149, components=[]` | `count=1, item=1149, components=[]` |
| final inventory slot count | `46` | `46` |
| final carried item | `empty` | `empty` |
| cursor observations | `0` | `0` |
| direct inventory observations | `0` | `0` |
| poison update/remove applied | `true / true` | `true / true` |
| block changed ACK sequences | `[29]` | `[29]` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet count | `5460` | `6560` |

Her iki oturumda vanilla başlangıç content'i 46 boş menu slotuyla local
player state'ine uygulandı. Console hazırlığı container slot `36` için state
`2` ve component ID `17` taşıdı. Honey bottle tüketimi sonrasında server state
`3` ile aynı slotu glass bottle registry ID `1149` olarak güncelledi; Rust
world kopyası state ID'yi, slotu ve carried boşluğunu aynı değerlere getirdi.

Ayrı bir vanilla pickup/state-id koşusunda (`P08mF`) başlangıç content state'i
`1`, server hazırlığı sonrası content state'i `2` ve stale click düzeltmesi
sonrası `container_set_slot` state'i `3`, menu slotu `36`, item registry ID'si
`1` (`stone`) ve count `8` olarak gözlendi. Bu slot observation da
`applied_to_world=true` idi; final local inventory map'i ilgili slotta
`stone x8` taşıdı.

Bu kontrollü akış cursor veya direct inventory packet'i üretmedi; bu iki
parser/handler yolu sentetik packet unit testleriyle ve world state API'siyle
doğrulandı. İki canlı koşuda `player_inventory_observations` ve cursor
observation sayısı `0` kaldı; bu, o dalların canlı senaryoda çalıştığı iddiası
olarak kullanılmaz.

Bu sonuç vanilla sunucusunun Rust packet tüketimini ve local player inventory
state uygulamasını gösterir; Java client eşitliği iddiası değildir.

## Açık kapsam

Custom menu açılışı ve slot haritası, creative ekranındaki remote-slot
senkronizasyonu, item component semantic etkileri, client-side click prediction,
inventory tick/stack kuralları, cursor/direct packet'leri üreten bağımsız canlı
senaryolar ve `client-parity` karşılaştırması açık kalır.
