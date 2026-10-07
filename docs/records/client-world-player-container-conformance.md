# P08.n — [server-conformance] LocalPlayer active container/menu state'i

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte
tutar. Kapsam, vanilla clientbound open-screen, container content, slot,
cursor ve close paketlerinin local player `ClientWorld` kopyasında sorgulanabilir
aktif menü state'ine bağlanmasıdır. Canlı kanıt sınıfı `server-conformance`'dır:
Rust probe'u Mojang vanilla 26.2 dedicated server'ına bağlanır ve server'ın
gönderdiği packet state'ini Rust world kopyasıyla birlikte ölçer. Aynı girdi
Java istemcisiyle karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundOpenScreenPacket.java:13-49` packet alanlarını container ID,
registry MENU type ve trusted Component title sırasıyla taşır. Play dispatch
tablosundaki packet ID `59`'dur. `ClientPacketListener.java:1394-1398`
open-screen handler'ı container ID ve title ile menu ekranı/menüsü oluşturma
yolunu başlatır.

`ClientboundContainerSetContentPacket.java:11-22` state ID, optional stack
listesi ve carried stack'i taşır. `ClientboundContainerSetSlotPacket.java:9-53`
container ID, state ID, signed slot ve optional stack taşır. Cursor packet'i
yalnız optional carried stack taşır. Bu yolların play packet ID'leri sırasıyla
content `18`, slot `20` ve cursor `96`'dır. `ClientPacketListener.java:1400-1455`
container ID `0` için `inventoryMenu`, eşleşen aktif container ID'si için
`containerMenu` kullanır; `handleSetPlayerInventory` doğrudan player inventory
index'ine yazar.

`ClientboundContainerClosePacket.java:9-43` packet içinde container ID taşır
ve play packet ID'si `17`'dir. Ancak `ClientPacketListener.java:1514-1517`
handler'ı bu ID'yi karşılaştırmadan `clientSideCloseContainer` çağırır.
`LocalPlayer.java:340-343` ve `Player.java:425-427` bu çağrının mevcut
container'ı player inventory menu'süne döndürdüğünü gösterir.

`MenuType.java:12-17` generic `9x1`–`9x6` menu tiplerini ardışık kaydeder;
vanilla menu registry ID'leri `0..5`'tir. `ChestMenu.java:17-39,49-58`
generic `9x3` tipinin ID `2` olduğunu ve üst container grid'inin `3 * 9 = 27`
slot taşıdığını gösterir. Aynı menu'nün standart player inventory bölümüyle
birlikte content listesi `63` slottur. `AbstractContainerMenu.java:618-630,769-775`
state ID ve carried state'in menu slotlarından ayrı tutulduğunu gösterir.

## Sözleşme

- Open-screen packet'i local player varken mevcut aktif container state'ini
  verilen container ID, menu type ID ve title'ın ham NBT bytes'ı ile oluşturur;
  önceki aktif container bu işlemde değiştirilir.
- Generic menu type ID `0..5`, üst grid için sırasıyla `9, 18, 27, 36, 45,
  54` slot anlamına gelir. Diğer menu type ID'leri için üst slot sayısı henüz
  çözülmemiş olarak tutulur.
- Container ID `0` content ve slot paketleri player inventory menu state'ine,
  aktif container ID'siyle eşleşen paketler aktif menu state'ine uygulanır.
  Content bütün listelenen slotları, state ID'yi ve carried item'i birlikte
  değiştirir; slot paketi tek slotu ve state ID'yi değiştirir.
- Cursor packet'i aktif menu varsa onun carried item'ini, açık menu yoksa
  player inventory carried state'ini değiştirir. Direct player inventory
  packet'i player inventory map'ine yazılır ve aktif menu slot map'ini
  değiştirmez.
- Container close packet'i payload içindeki ID ne olursa olsun mevcut aktif
  container'ı kapatır; ID observation'da korunur. Açık container yoksa world
  state'i değişmez.
- Local player yoksa packet observation korunur fakat world state'e uygulanmaz.
  Yeni local player oluşturulduğunda aktif container ve player inventory
  kopyaları constructor varsayılanlarına döner.

## Rust doğrulaması

`ClientContainerState`, container ID, menu type ID, raw title NBT, çözülebilen
üst slot sayısı, content slot count, state ID, slot map ve carried item'i
sorgulanabilir olarak tutar. `ClientWorld` open-screen, close, content, slot ve
cursor handler'larını aktif local player state'ine bağlar. Open-screen parser'ı
container/menu VarInt'lerini ve tek NBT title root'unu packet sonuna kadar
okur; title'ın tüketilen raw bytes'ı korunur. Content, slot ve direct inventory
parser'ları önceki inventory stack modelini kullanır.

`examples/world_player_container_probe.rs` vanilla server'ın hazırladığı
generic chest'e `use_item_on` gönderir, open-screen ve sonrasındaki 63 slotluk
content'i kontrol eder ve aynı değerleri `ClientWorld::local_player` üzerinden
sorgular. World unit test'i open/content/slot/cursor/close ve respawn sonrası
container temizliğini kapsar. Canlı probe, server'ın chest'i hazırlamasına
zaman tanımak için etkileşimden önce 20 saniye bekler.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
başlangıç oyun modu creative, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Her oturumda vanilla server konsolundan oyuncunun destek bloğuna bir chest
konuldu ve chest'in slot `0`'ına `stone x7` verildi. Rust probe daha sonra
aynı oyuncu konumundan chest'e `use_item_on` gönderdi.

İki bağımsız geçerli vanilla oturumunda gözlenen sonuçlar:

| Gözlem | `P08nD` | `P08nE` |
| --- | ---: | ---: |
| login player ID | `5011` | `5145` |
| open-screen | `container=1, menu_type=2, applied=true` | `container=1, menu_type=2, applied=true` |
| title | `translate=container.chest` | `translate=container.chest` |
| custom content | `state=1, items=63, applied=true` | `state=1, items=63, applied=true` |
| custom slot 0 | `count=7, item=1 (stone)` | `count=7, item=1 (stone)` |
| Rust active container | `id=1, type=2, top=27, slots=63, state=1, slot0=7x1` | `id=1, type=2, top=27, slots=63, state=1, slot0=7x1` |
| close observations | `0` | `0` |
| chunk packet / decoded / batch finished | `47 / 47 / 13` | `47 / 47 / 13` |
| received packet count | `4390` | `4817` |

Her iki canlı koşuda open-screen, content ve world active-container state'i
uygulandı. Title NBT'sinin ham bytes'ı observation'da korunarak vanilla'nın
`container.chest` çeviri bileşeni ölçüldü. Bu koşular menu açıkken sona erdiği
için canlı close observation oluşmadı; close parser'ı, ID'yi göz ardı eden
world handler'ı ve close/respawn temizliği unit testte doğrulandı.

Bu sonuç vanilla sunucusunun Rust packet tüketimini ve local player aktif menu
state uygulamasını gösterir; Java client eşitliği iddiası değildir.

## Açık kapsam

Özel menu tiplerinin slot haritaları, menu click işlemleri ve client-side
prediction, creative ekranının remote-slot dalı, semantic item component
etkileri, GUI ekranının oluşturulması, menu kapanışının gerçek ekran/input
akışı ve `client-parity` karşılaştırması açık kalır.
