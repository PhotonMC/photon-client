# P08.ag — [server-conformance] Entity custom-name presence semantic state

Bu kayıt, `ClientboundSetEntityDataPacket` içindeki Java entity custom-name
optional component alanının Rust `ClientWorld` state'ine semantik olarak
uygulanmasını kapsar. Kanıt sınıfı `server-conformance`'dır: Rust packet akışı
Mojang vanilla 26.2 dedicated server'a bağlanır. Java istemcisiyle aynı girdide
state karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Entity.java:264-266`, custom name'i ID `2` `Optional<Component>` alanı olarak
tanımlar. `Entity.java:315-317`, yeni entity'de bu alanı boş başlatır;
`Entity.java:3293-3304` setter, getter ve `hasCustomName()` yolunda Optional'ın
varlığını kullanır. NBT yükleme yolu `Entity.java:2180-2181`, `CustomName`
component'ini entity state'ine yükler.

`Entity.java:2955-2959`, custom name varsa entity `getName()` sonucunun bu
component'ten geldiğini gösterir. `MobRenderer.java:14-16`, custom name'in
visibility alanı kapalı olsa bile entity crosshair üzerinde seçiliyken
`hasCustomName()` sonucunu name label kararında kullanır; `MobRenderer.java:24-26`
custom name component'ini özel isim kontrollerinde de okur.

`EntityDataSerializers.java:53-54,164-172`, `OPTIONAL_COMPONENT` serializer'ının
ID `6`, `BOOLEAN` serializer'ının ID `8` olduğunu verir. `ByteBufCodecs.java:373-387`,
optional alanın presence byte'ı ve mevcutsa component payload'ı taşıdığını
gösterir. Trusted component payload'ı NBT root olarak kodlanır. Metadata
kayıtlarının ID, serializer, değer ve `255` sonlandırıcısı biçimi
`ClientboundSetEntityDataPacket.java:21-37` içindedir; dirty kayıtların yalnız
gönderilen alanları değiştirmesini `SynchedEntityData.java:74-85,108-125`
belirtir.

## Sözleşme

- Geçerli metadata içinde ID `2` ve `OPTIONAL_COMPONENT` serializer ID `6`
  varsa presence byte'ı `0` `has_custom_name` değerini false, sıfır dışı
  presence ve geçerli NBT component'i true yapar.
- ID `2` güncellemesi custom-name visibility ID `3` alanından bağımsızdır;
  bir alanın gönderilmemesi diğer alanın önceki state'ini korur.
- Yeni `ClientEntity` ve yeni `ClientPlayerState` custom-name presence
  değerini false ile başlatır.
- Optional component temizlendiğinde `has_custom_name` false olur; component'in
  metin ve style içeriği bu dilimde ayrı bir state olarak tutulmaz.
- Local player yeniden kurulmasında `dataToKeep` bit 2 varsa custom-name presence
  değeri mevcut entity data ile birlikte korunur; bit 2 yoksa yeni oyuncu false
  ile başlar.

## Rust doğrulaması

`custom_name_present_from_metadata`, ID `2` `OPTIONAL_COMPONENT` kaydının
presence byte'ını ve mevcut component'in NBT root'unu tüketir. `ClientEntity::has_custom_name`
ve `ClientPlayerState::has_custom_name` değerleri world kopyasından
sorgulanabilir. Önceki P08.af state'iyle birlikte component presence ve
custom-name visibility bağımsız güncellenir.

World unit testi varsayılan false state'ini, NBT component varlığını, visibility
alanından bağımsız temizlemeyi ve component temizlendiğinde false'a dönmeyi
doğrular. Local player respawn testi `dataToKeep` bit 2 ile presence state'inin
korunmasını ve boş maskeyle false değerine dönülmesini kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken custom name component'i olan ve konumunu sabitlemek
için `NoGravity:1b` taşıyan bir minecart oluşturuldu. Visibility alanı
ayarlanmadığı için iki entity state'inin bağımsızlığı da gözlendi:

```text
cargo run --locked --offline --example world_entity_custom_name_presence_probe -- 127.0.0.1:25568 P08NameL2
summon minecraft:minecart 31 90 -36 {CustomName:'{"text":"P08NamedMinecart"}',NoGravity:1b}
```

Vanilla sunucusu summon komutunu kabul etti. Probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `119` |
| `custom_name_present` | `true` |
| `custom_name_visible` | `false` |
| `should_show_name` | `false` |
| `entity_type_id` | `85` |
| `entity_position` | `(31.5, 90.0, -35.5)` |
| `entity_metadata_length` | `37` |
| `world_entity_count` | `44` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `10250` |

Custom component içeren metadata packet'inin ID `2` presence byte'ı ve NBT
payload'ı ile world map'inde bulunması, vanilla sunucunun optional component
akışının Rust semantic state'e ulaştığını gösterir. Visibility false kalırken
presence true gözlemi iki alanın bağımsız tutulduğunu doğrular. Bu sonuç Java
istemcisiyle parity iddiası değildir.

## Açık kapsam

Custom component'in metin/style ve tıklama davranışının world kopyasında
tutulması, entity name çözümü, MobRenderer uzaklık/crosshair seçimi ve gerçek
name label render/font ölçümü, Player görünürlük override'ının renderer'a
bağlanması, diğer component serializer'ları ve `client-parity` karşılaştırması
açık kalır.
