# P08.af — [server-conformance] Entity custom-name visibility semantic state

Bu kayıt, `ClientboundSetEntityDataPacket` içindeki Java entity custom-name
visibility alanının Rust `ClientWorld` state'ine semantik olarak uygulanmasını
kapsar. Kanıt sınıfı `server-conformance`'dır: Rust packet akışı Mojang vanilla
26.2 dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Entity.java:264-268`, custom name'i ID `2` optional component, görünürlük
alanını ID `3` boolean olarak tanımlar. `Entity.java:315-317`, yeni entity için
custom name'i boş ve `DATA_CUSTOM_NAME_VISIBLE` değerini `false` başlatır.
`Entity.java:3293-3313`, custom name ve görünürlük accessor'larını; `Entity.java:3368-3369`
ise temel `shouldShowName()` sonucunun custom-name visibility alanından geldiğini
gösterir. Entity NBT yükleme yolu `Entity.java:2180-2181` görünürlük alanını
`CustomNameVisible` tag'inden okur.

Player bu temel sonucu override eder: `Player.java:1637-1639` her zaman `true`
döndürür. Mob renderer `MobRenderer.java:14-16`, uzaklık koşulundan sonra bu
sonucu veya crosshair üzerindeki custom name istisnasını kullanır.

`EntityDataSerializers.java:53-54,164-172`, `OPTIONAL_COMPONENT` serializer'ını
ID `6`, `BOOLEAN` serializer'ını ID `8` olarak verir. `ByteBufCodecs.java:56-66`
boolean değerinin tek byte, `ByteBufCodecs.java:373-387` optional değerinin
presence byte'ı ve varsa component payload'ı taşıdığını gösterir. Component
payload'ı trusted codec üzerinden NBT root olarak kodlanır. Metadata kayıtlarının
ID, serializer, değer ve `255` sonlandırıcısı biçimi
`ClientboundSetEntityDataPacket.java:21-37` içindedir; dirty kayıtların yalnız
gönderilen alanları değiştirmesini `SynchedEntityData.java:74-85,108-125`
belirtir.

## Sözleşme

- Geçerli metadata içinde ID `3` ve `BOOLEAN` serializer ID `8` varsa byte
  değeri sıfırsa `custom_name_visible` false, sıfır dışıysa true olur.
- ID `3`, ID `2` `OPTIONAL_COMPONENT` custom name kaydından sonra gelebilir;
  optional değer presence byte'ı ve varsa NBT component root'u tüketilir.
- ID `3` tek başına veya daha önce desteklenen shared flags, air supply, silent,
  no-gravity, pose ve freeze progress kayıtlarıyla birlikte gönderilebilir;
  gönderilmeyen alanlar önceki değerlerini korur.
- Yeni `ClientEntity` ve yeni `ClientPlayerState` görünürlük değerini `false`
  ile başlatır.
- Genel entity için `should_show_name()` custom-name visibility değerini
  döndürür; vanilla player entity'si için Player override'ı nedeniyle sonuç
  her zaman true'dur.
- Local player yeniden kurulmasında `dataToKeep` bit 2 varsa görünürlük değeri
  mevcut entity data ile birlikte korunur; bit 2 yoksa yeni oyuncu görünürlük
  değerini false başlatır.

## Rust doğrulaması

`metadata_boolean_value`, ID `3` alanına ulaşana kadar Java metadata sırasındaki
BYTE, INT, BOOLEAN, POSE ve OPTIONAL_COMPONENT kayıtlarını tüketir. Optional
component present ise NBT root'u ayrıştırarak sonraki metadata kaydının doğru
başlangıcını korur. `ClientEntity::is_custom_name_visible` ve
`ClientPlayerState::is_custom_name_visible` state'i world kopyasından
sorgulanabilir; `should_show_name` entity türü için Java base/Player ayrımını
taşır.

World unit testi varsayılan false state'ini, custom component'ten sonra gelen
ID `3` true/false geçişini ve Player override sonucunu doğrular. Local player
respawn testi `dataToKeep` bit 2 ile görünürlük state'inin korunmasını, boş
maskeyle false değerine dönülmesini ve player'ın `should_show_name()` sonucunun
true kalmasını kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken custom name component'i ve `CustomNameVisible:1b` alanı
olan, ayrıca konumunu sabitlemek için `NoGravity:1b` taşıyan bir armor stand
oluşturuldu:

```text
cargo run --locked --offline --example world_entity_custom_name_probe -- 127.0.0.1:25568 P08NameL1
summon minecraft:armor_stand 25 90 -36 {CustomName:'{"text":"P08NameVisible"}',CustomNameVisible:1b,NoGravity:1b}
```

Vanilla sunucusu summon komutunu kabul etti. Probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `198` |
| `custom_name_visible` | `true` |
| `should_show_name` | `true` |
| `entity_type_id` | `5` |
| `entity_position` | `(25.5, 90.0, -35.5)` |
| `entity_metadata_length` | `44` |
| `world_entity_count` | `45` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `10948` |

Custom component içeren metadata packet'inin ID `3` boolean alanına kadar
tüketilmesi ve görünürlük ile `should_show_name` sonuçlarının world map'inde
sorgulanması, vanilla sunucunun optional component + BOOLEAN metadata akışının
Rust semantic state'e ulaştığını gösterir. Bu sonuç Java istemcisiyle parity
iddiası değildir.

## Açık kapsam

Custom name component'inin metin/style state olarak world kopyasında tutulması,
MobRenderer'ın uzaklık ve crosshair istisnası, gerçek name label render/font
ölçümü, Player görünürlük override'ının renderer'a bağlanması, diğer metadata
serializer'ları ve `client-parity` karşılaştırması açık kalır.
