# P08.r — [server-conformance] Vanilla container menu slot haritası

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte
tutar. Kapsam, clientbound open-screen ile gelen vanilla MENU registry ID'sini
bilinen menu türünün menu-özel slot sayısına bağlamaktır. Canlı kanıt sınıfı
`server-conformance`'dır: Rust probe'u Mojang vanilla 26.2 dedicated server'ına
bağlanır ve server'ın hopper menu paketini `ClientWorld` içindeki aktif menu
state'iyle karşılaştırır. Java client ile aynı girdinin karşılaştırması
yapılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`MenuType.java:12-36` vanilla menu registry'sini sabit kayıt sırasıyla kurar.
Generic chest türleri ID `0..5`, dispenser `6`, crafter `7`, anvil `8`, beacon
`9`, blast furnace `10`, brewing stand `11`, crafting `12`, enchantment `13`,
furnace `14`, grindstone `15`, hopper `16`, lectern `17`, loom `18`, merchant
`19`, shulker box `20`, smithing `21`, smoker `22`, cartography table `23` ve
stonecutter `24` olur.

`ChestMenu.java:49-64` generic türlerde `9 * rows` container slotu ekler.
`DispenserMenu.java:20-28` ve `CrafterMenu.java:22-53` sırasıyla 9 dispenser
slotu ve 9 crafting slotu ile crafter result slotunu kurar. `ItemCombinerMenu.java:31-48`
input slotlarını result slotuyla birlikte ekleyip standart player inventory
slotlarını ekler; anvil ve smithing tanımları bu ortak yolu kullanır.
`BeaconMenu.java:46-54`, `BrewingStandMenu.java:41-55`, `HopperMenu.java:17-28`,
`LecternMenu.java:21-32`, `LoomMenu.java:60-108`, `MerchantMenu.java:37-45`,
`ShulkerBoxMenu.java:16-31`, `CartographyTableMenu.java:45-90` ve
`StonecutterMenu.java:49-85` kendi menu-özel slot sayılarını açıkça ekler.
`AbstractFurnaceMenu.java:24-71`, `EnchantmentMenu.java:51-74` ve
`GrindstoneMenu.java:44-106` kalan cooking, enchantment ve grindstone
türlerinin slotlarını kurar. `CraftingMenu.java:37-45` crafting result ve 3x3
grid'i ekler.

`AbstractContainerMenu.java:68-90` standart player inventory'nin 27 main ve
9 hotbar slotundan oluştuğunu gösterir. `CrafterMenu` result slotunu bu 36
inventory slotundan sonra eklediği için menu-özel slot sayısı `10` olsa da
result packet slot index'i `45` olarak kalır; raw slot map'i bu index ayrımını
korur.

## Vanilla menu tablosu

Rust, bilinen registry ID'leri için aşağıdaki menu-özel slot sayısını
`ClientContainerState::top_slot_count` üzerinden sorgulanabilir hale getirir.
Content packet'inin gerçek toplam slot sayısı ise packet listesinden ayrıca
`slot_count` olarak tutulur.

| Registry ID | Vanilla menu key | Menu-özel slot sayısı |
| ---: | --- | ---: |
| 0 | `generic_9x1` | 9 |
| 1 | `generic_9x2` | 18 |
| 2 | `generic_9x3` | 27 |
| 3 | `generic_9x4` | 36 |
| 4 | `generic_9x5` | 45 |
| 5 | `generic_9x6` | 54 |
| 6 | `generic_3x3` | 9 |
| 7 | `crafter_3x3` | 10 |
| 8 | `anvil` | 3 |
| 9 | `beacon` | 1 |
| 10 | `blast_furnace` | 3 |
| 11 | `brewing_stand` | 5 |
| 12 | `crafting` | 10 |
| 13 | `enchantment` | 2 |
| 14 | `furnace` | 3 |
| 15 | `grindstone` | 3 |
| 16 | `hopper` | 5 |
| 17 | `lectern` | 1 |
| 18 | `loom` | 4 |
| 19 | `merchant` | 3 |
| 20 | `shulker_box` | 27 |
| 21 | `smithing` | 4 |
| 22 | `smoker` | 3 |
| 23 | `cartography_table` | 3 |
| 24 | `stonecutter` | 2 |

Tanımsız veya gelecekte eklenmiş registry ID'leri `None` olarak kalır; bu
durum bilinmeyen bir menu'nün yanlış bir slot düzeniyle sessizce açılmasını
önler. Slot packet index'leri ayrıca korunur ve menu slotları için henüz
click, recipe, data-slot veya GUI semantiği uygulanmaz.

## Rust doğrulaması

`menu_top_slot_count` vanilla registry'deki 25 ID'nin tamamını tek parametrik
unit testte kontrol eder; `-1` ve `25` bilinmeyen ID'leri `None` sonucunu
doğrular. `ClientContainerState` open-screen sırasında bu tabloyu kullanır,
content packet'i geldiğinde gerçek list length'ini `slot_count` alanına yazar
ve her slotu kendi packet index'iyle saklar.

`examples/world_player_container_menu_map_probe.rs` mevcut vanilla container
probe akışını hopper için kullanır. Server oyuncunun destek bloğunu hopper'a
çevirir ve hopper slot `0`'a `stone x7` yerleştirir; Rust probe 20 saniyelik
hazırlık aralığından sonra aynı bloğa boş main hand ile `use_item_on` yollar.
Probe open-screen menu ID'sini `16`, content listesini `41`, slot `0`'ı
`stone x7` ve Rust world state'ini `top=5, slots=41` olarak kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
başlangıç oyun modu creative, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Her oturumda oyuncunun gerçek başlangıç konumu server logundan alındı.
Oyuncunun destek bloğuna hopper konuldu, hopper slot `0`'a `stone x7`
yerleştirildi ve Rust probe'un gecikmeli `use_item_on` paketi beklendi.

İki bağımsız geçerli vanilla oturumunda gözlenen sonuçlar:

| Gözlem | `P08rA` | `P08rB` |
| --- | ---: | ---: |
| login player ID | `615` | `781` |
| play login / player loaded | `true / true` | `true / true` |
| open-screen | `container=1, menu_type=16, applied=true` | `container=1, menu_type=16, applied=true` |
| title | `translate=container.hopper` | `translate=container.hopper` |
| hopper content | `state=1, items=41, applied=true` | `state=1, items=41, applied=true` |
| hopper slot 0 | `count=7, item=1 (stone)` | `count=7, item=1 (stone)` |
| Rust active container | `id=1, type=16, top=5, slots=41, state=1, slot0=7x1` | `id=1, type=16, top=5, slots=41, state=1, slot0=7x1` |
| chunk packet / decoded / batch finished | `42 / 42 / 11` | `43 / 43 / 12` |
| received packet count | `4074` | `7029` |

Her iki canlı koşuda non-generic hopper menu'sü open-screen ve content
paketlerinden Rust active-container state'ine uygulandı. 25 registry ID'sinin
tam slot tablosu Java source oracle'ı ve Rust parametrik unit test ile
kapatıldı; canlı koşu bunun içinden değişken menü branch'i olan ID `16` için
wire kabulü ve world uygulamasını gösterir.

Bu sonuç vanilla sunucusunun Rust packet tüketimini ve menu slot metadata
uygulamasını gösterir; Java client eşitliği iddiası değildir.

## Açık kapsam

Menu slotlarının tam GUI koordinatları ve crafter gibi dağınık index haritaları,
container click/prediction, recipe/data-slot güncellemeleri, semantic item ve
block entity state'i, merchant/lectern özel payload'ları, gerçek ekran/input
akışı ve `client-parity` karşılaştırması açık kalır.
