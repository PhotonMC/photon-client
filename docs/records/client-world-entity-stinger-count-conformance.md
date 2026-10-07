# P08.ao — [server-conformance] Entity living stinger count semantic state

Bu kayıt, Java LivingEntity arı iğnesi sayısı alanının Rust ClientWorld
state'ine uygulanmasını kapsar. Kanıt sınıfı server-conformance'dır: Rust
packet akışı Mojang vanilla 26.2 dedicated server'a bağlanır. Java istemcisiyle
aynı girdide state karşılaştırılmadığı için bu kayıt client-parity değildir.

## Java kaynak incelemesi

LivingEntity.java:176-186, arrow count alanından ayrı bir
DATA_STINGER_COUNT_ID synchronized alanını tanımlar. Alan ID 13 ve INT
serializer'ı kullanır; LivingEntity.java:322-329 başlangıç değerini 0 yapar.
LivingEntity.java:2008-2013 getter/setter'ın bu synchronized değeri
okuduğunu ve yazdığını, LivingEntity.java:2769-2779 ise server tick'inde
stinger sayısının zamanla azaltılabildiğini gösterir.

Bee.java:230, başarılı sting hasarından sonra hedef yaşayan entity ise stinger
sayısını bir artırır. Client render tarafında AvatarRenderer.java:17,173
BeeStingerLayer'ı ekler ve sayıyı render state'e taşır;
BeeStingerLayer.java:12-27 bu değeri stuck bee stinger adedi olarak kullanır.

EntityDataSerializers.java:53-54,164-172, INT serializer ID'sini 1 olarak
kaydeder. ByteBufCodecs.java:106-113 signed VarInt codec'ini tanımlar.
SynchedEntityData.java:74-85,108-125 yalnız dirty alanların metadata'ya
girmesini, ClientboundSetEntityDataPacket.java:21-37 ise metadata kaydının ID,
serializer, değer ve 255 sonlandırıcısından oluştuğunu belirler.

Arrow count ile aynı primitive codec kullanılsa da ID 13 ayrı bir Java alanı ve
ayrı bir BeeStingerLayer dalıdır; bu dilim arrow count için yeni sabit varyantı
değildir.

## Sözleşme

- Geçerli metadata içinde ID 13 ve INT serializer ID 1 varsa signed VarInt
  değer i32 olarak okunur ve ClientEntity::stinger_count üzerinden sorgulanır.
- Yeni entity ve local player stinger count değeri 0 ile başlar. Yalnız ID 13
  içeren metadata packet'i [13, 1, value, 255] biçiminde uygulanabilir.
- Metadata yalnız gönderilen alanları günceller. ID 13 içermeyen sonraki packet
  önceki sayıyı korur; yanlış serializer veya eksik değer state'i değiştirmez.
- Local player metadata'sı aynı değeri player world kopyasına uygular. Respawn
  KEEP_ENTITY_DATA maskesiyle entity data korunduğunda stinger count korunur;
  boş keep maskesi yeni player varsayılanı olan 0 değerine döner.
- Bu dilim bee AI hedef seçimini, sting hasarını, poison effect'ini, stinger
  decay zamanlamasını, model/texture render'ını veya Java client state'ini
  karşılaştırmayı uygulamaz.

## Rust doğrulaması

living_count_from_metadata, Java'nın ortak INT/VarInt metadata yolunu hedef
data ID'siyle parametreleyerek okur. Arrow ve stinger alanları Rust state'inde
ayrı tutulur; ClientWorld metadata uygulaması remote entity ve local player
dallarında ID 13 değerini günceller. Entity-data korumalı local player yeniden
oluşturmasında değer kopyalanır.

World unit testleri entity'de 0 başlangıcını, 2 değerini, kısmi metadata
sonrasında korunmasını ve yanlış serializer'ın reddedilmesini kontrol eder.
Local player testi metadata değerini, KEEP_ENTITY_DATA ile korunmasını ve boş
keep maskesiyle 0 değerine dönmesini kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed 776, creative
test modu, view-distance=5, simulation-distance=5, 127.0.0.1:25568

Probe bağlantısı açıkken oyuncu normal zorlukta survival moda alındı. Vanilla
bee hedef dalını çalıştırmak için universal_anger açıldı; eski AngerTime etiketi
yerine sunucunun güncel mutlak anger_end_time alanı kullanıldı. Oyun zamanı
14549 iken oyuncunun bulunduğu konuma NoAI kapalı ve henüz sting yapmamış bir
bee çağrıldı:

    cargo run --locked --offline --example world_player_stinger_count_probe -- 127.0.0.1:25568 P08Sting3
    difficulty normal
    gamemode survival P08Sting3
    gamerule universal_anger true
    time query gametime
    execute at P08Sting3 run summon minecraft:bee ~ ~ ~ {anger_end_time:15749L,NoAI:0b,HasStung:0b}

Bee oyuncuya başarılı biçimde sting yaptı; vanilla sunucusunun gönderdiği ID
13 metadata kaydı Rust ClientWorld içindeki local player state'ine uygulandı:

| Gözlem | Sonuç |
| --- | ---: |
| play_login | true |
| player_loaded_sent | true |
| entity_metadata_packet_count | 527 |
| stinger_count | 1 |
| local_player_id | 1457 |
| local_player_position | (21.5, 77.0, -31.5) |
| world_entity_count | 79 |
| world_chunk_count | 157 |
| received_packet_count | 14165 |

Bu sonuç vanilla bee attack dalının yaşayan hedefte stinger count'u artırdığını
ve Rust'ın gelen entity metadata'yı sorgulanabilir player state'ine taşıdığını
gösterir. Sonuç Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Stinger count'un server tick'iyle azalması, birden fazla sting, bee poison
effect'i, bee AI ve hedef geçişleri, remote living entity state'i, player/bee
render katmanı, stinger model/texture, gerçek istemci tick'i ve client-parity
karşılaştırması açık kalır.
