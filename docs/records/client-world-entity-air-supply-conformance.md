# P08.aa — [server-conformance] Entity air supply semantic state

Bu kayıt, `ClientboundSetEntityDataPacket` içindeki Java entity air supply
alanının Rust `ClientWorld` state'ine semantik olarak uygulanmasını kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Entity.java:189`, temel toplam air supply değerini `300` olarak tanımlar.
`Entity.java:263-271,313`, ortak flags alanından sonra
`DATA_AIR_SUPPLY_ID` alanını ID `1` ve `INT` serializer'ı ile kurar; entity
kurucusu bu alanı `getMaxAirSupply()` sonucu ile başlatır.
`Entity.java:2800-2809`, temel entity için maksimum değeri `300` döndürür ve
air supply okuma/yazmasını entity data üzerinden yapar.

`EntityDataSerializers.java:53-54,148-166`, `BYTE` serializer'ının ID `0`,
`INT` serializer'ının ID `1` olduğunu kayıt sırasından gösterir. Bu nedenle
metadata kaydı ID `1`, serializer ID `1` ve signed VarInt değerinden oluşur.
`SynchedEntityData.java:74-85,108-125`, dirty değerlerin ID sırasıyla
paketlendiğini ve gelen packet'te yalnız listelenen alanların atandığını
belirtir. `ClientboundSetEntityDataPacket.java:21-37`, kayıtların ID byte'ı,
serializer VarInt'i, değer ve `255` sonlandırıcısından oluştuğunu gösterir.

`LivingEntity.java:436-456`, living entity su altında nefes alamıyorsa air
supply'ı her tick azaltır; su dışındayken eksik değeri `4` artırarak maksimuma
ulaştırır ve dirty entity data üretir. `LivingEntity.java:587-600`, varsayılan
azaltma adımını bir ve artış adımını dört olarak verir. Entity türleri bu
değerleri değiştirebilir; örneğin `Axolotl.java:276` maksimumu `6000`,
`Dolphin.java:194` maksimumu `4800` yapar. Bu dilim temel `300` değeri ve
oyuncu yolunu kapsar.

## Sözleşme

- Geçerli metadata içinde ID `1` ve `INT` serializer ID `1` varsa signed VarInt
  değeri entity veya local player air supply state'ine yazılır.
- ID `0` shared flags ile ID `1` air supply aynı metadata gövdesinde
  gönderilebilir; iki alan da uygulanır. Yalnızca bir alan gönderildiğinde
  diğer alanın önceki değeri korunur.
- Yeni `ClientEntity` ve yeni `ClientPlayerState` temel air supply değerini
  `300` ile başlatır.
- Local player yeniden kurulumunda `dataToKeep` bit 2 varsa air supply mevcut
  entity data ile birlikte korunur; bit 2 yoksa yeni oyuncu `300` ile başlar.
- Metadata gövdesindeki signed VarInt değerleri, Java serializer'ının wire
  değerleri olarak işaretli biçimde saklanır. Raw metadata ayrıca önceki
  davranışta olduğu gibi sorgulanabilir kalır.

## Rust doğrulaması

`metadata_int_value`, Java'nın ID sırasına göre ID `0` BYTE kaydını atlayıp ID
`1` INT kaydını çözer; bilinmeyen serializer veya eksik değer air supply
state'ini değiştirmez. `ClientEntity::air_supply` ve
`ClientPlayerState::air_supply` değerleri world kopyasından sorgulanabilir.

World unit testi ID `1` INT metadata'sını tek başına, shared flags ile birlikte,
signed `-1` değerinde ve sonraki yalnız flags güncellemesinin air supply'ı
korumasında doğrular. Local player respawn testi `dataToKeep` bit 2 ile `250`
değerinin korunmasını, boş maskeyle `300` değerine dönülmesini kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
başlangıç creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı kurulduktan sonra oyuncu için canlı su altı koşulu hazırlandı.
Sunucunun başlangıç modu creative olduğu için oyuncu Survival'a geçirildi; su
iki blok yüksekliğinde ve oyuncunun göz seviyesini kapsayacak şekilde
`(24,80,-33)`–`(24,81,-33)` aralığına yazıldı, ardından oyuncu
`(24.5,80,-32.5)` konumuna taşındı:

```text
cargo run --locked --offline --example world_entity_air_supply_probe -- 127.0.0.1:25568 P08AirL2
gamemode survival P08AirL2
fill 24 80 -33 24 81 -33 minecraft:water
tp P08AirL2 24.5 80 -32.5
```

Sunucu komutları oyuncu oturumu açıkken uygulandı. Probe 30 saniyelik metadata
drain penceresinde air supply değişikliklerini tüketti. Çıktı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `618` |
| `air_supply` | `-6` |
| `max_air_supply` | `300` |
| `world_entity_count` | `78` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `13616` |

Air supply'ın `300`'den düşmesi ve son değerin world local player state'inden
okunması, vanilla sunucunun ID `1` INT metadata akışının Rust semantic state'e
ulaştığını gösterir. Bu sonuç Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Entity türlerinin `getMaxAirSupply()` override'ları ve türe özel breathing/
decrease davranışı, su ve bubble-column algısı, water-breathing/oxygen bonusu,
drowning damage/event akışı, air refill tick yürütmesi, air HUD/render state'i,
remote living entity yaşam döngüsü, diğer metadata serializer'ları ve
`client-parity` karşılaştırması açık kalır.
