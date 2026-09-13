# P08.h — [server-conformance] LocalPlayer yeniden kurulumu ve `dataToKeep`

Bu kayıt, vanilla respawn packet'inin local player state'ine uygulanan kısmını
kapsar. Canlı kanıt sınıfı `server-conformance`'dır: Rust probe'u Mojang
vanilla 26.2 dedicated server'ına bağlanır, sunucu konsolundan dimension
değişimi tetiklenir ve Rust world kopyasındaki local player sorgulanır. Aynı
girdi Java istemcisiyle karşılaştırılmadığı için bu kayıt `client-parity`
değildir.

## Java kaynak incelemesi

`ClientboundRespawnPacket.java:8-38`, `dataToKeep` içindeki
`KEEP_ATTRIBUTE_MODIFIERS=1` ve `KEEP_ENTITY_DATA=2` bitlerini tanımlar;
`shouldKeep` yalnız ilgili mask bitini test eder.

`ClientPacketListener.java:1247-1355`, respawn'da yeni `LocalPlayer`
oluşturulduğunu ve eski oyuncunun entity ID'sinin yeni nesneye verildiğini
gösterir. Entity-data biti açıksa non-default entity data, delta movement ve
yaw/pitch yeni oyuncuya aktarılır. Bit kapalıysa oyuncu reset konumuna alınır ve
yaw `-180` değerine ayarlanır. Attribute biti açıksa tüm attribute değerleri,
kapalıysa yalnız base değerler aktarılır.

`ClientboundPlayerPositionPacket.java:12-30` ile
`PositionMoveRotation.java:8-18,39-72`, ilk konum düzeltmesinin position,
delta movement ve rotasyon alanlarını taşır. Rust state modeli bu paketin
relative çözülmüş konumunu, delta movement'ını ve yaw/pitch değerlerini
sorgulanabilir local player state'ine yazar.

## Sözleşme

- Login packet'indeki player ID, `ClientWorld` içinde local player state'ini
  başlatır.
- Clientbound player-position düzeltmesi, relative flag'ler çözüldükten sonra
  local player konumunu, hareketini ve rotasyonunu günceller. Bu paket gelene
  kadar yeni local player'ın başlangıç konumu modelde sıfırdır.
- Local player ID'sine gelen entity metadata, local player'ın ham entity-data
  state'ine yazılır; diğer entity metadata akışı mevcut entity map'inde kalır.
- Respawn her zaman aynı entity ID ile yeni bir local player nesli oluşturur.
  Dimension world storage'ı temizlense bile ayrı local player state'i korunur.
- `dataToKeep` bit 2 açıksa önceki local player'ın entity data, hareket ve
  rotasyon state'i yeni nesle taşınır. Bit kapalıysa bu state'ler yeni default
  değerlerine döner.
- Bit 1'in attribute base/modifier aktarımı [P08.j kaydında](client-world-player-attributes-conformance.md)
  uygulanmıştır; bu belge bit 2 entity state'i ve ortak local player yeniden
  kurulumunu kapsar.

## Rust doğrulaması

`ClientWorld::local_player`, local player ID'sini, nesil sayacını, position,
motion, rotation ve ham entity data state'ini sorgular. Login, player-position,
entity-metadata ve respawn packet dalları bu state'e bağlanır.

Unit testler player ID'sinin dimension replacement sonrasında korunmasını,
respawn nesil artışını, bit 2 açıkken motion/rotation/metadata aktarımını ve
maskesiz respawn'da bu state'lerin temizlenmesini doğrular. Player-position
observation'ı da local world uygulama sonucunu kaydeder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Probe çalışırken vanilla server konsolunda her oturum için şu komut çalıştırıldı:

```text
execute as <probe-user> in minecraft:the_nether run tp @s 0 80 0
```

Rust probe:

```text
cargo run --locked --offline --example world_player_respawn_probe -- 127.0.0.1:25568 <probe-user>
```

İki bağımsız vanilla oturumunda gözlenen sonuçlar:

| Gözlem | `P08hA` | `P08hB` |
| --- | ---: | ---: |
| login player ID | `1756` | `1988` |
| respawn packet count | `1` | `1` |
| respawn dimension | `minecraft:the_nether` | `minecraft:the_nether` |
| `data_to_keep` | `3` | `3` |
| `keep_entity_data` | `true` | `true` |
| `keep_attribute_modifiers` | `true` | `true` |
| local player ID after respawn | `1756` | `1988` |
| local player recreated | `true` | `true` |
| local player generation | `1` | `1` |
| post-correction position | `(0.5, 80.0, 0.5)` | `(0.5, 80.0, 0.5)` |
| post-correction motion | `(0.0, 0.0, 0.0)` | `(0.0, 0.0, 0.0)` |
| post-correction rotation | `(0.0, 0.0)` | `(0.0, 0.0)` |
| local player entity data present | `true` | `true` |
| dimension changed | `true` | `true` |
| final world chunk count | `157` | `157` |

Her iki oturumda login ID'si respawn sonrasında değişmedi, fakat local player
nesli `0`'dan `1`'e çıktı. İlk player-position düzeltmesi world state'ine
uygulandı ve Nether geçişinden sonra local state metadata ile birlikte
sorgulanabildi. `cleared_entity_count` sunucu tick zamanlamasına göre
değiştiği için eşitlik ölçütü yapılmadı.

Güncellenmiş probe ile yapılan üçüncü oturumda (`P08hC`) ayrıca
`player_position_observation_count=2` ve
`player_position_all_applied_to_world=true` gözlendi.

Bu sonuç packet kabulü ve Rust world state uygulamasını gösterir; Java client
eşitliği iddiası değildir.

## Açık kapsam

Same-dimension ölüm respawn'ı ve `dataToKeep=0` canlı senaryosu
[P08.i kaydında](client-world-player-death-respawn-conformance.md)
doğrulanır. Effective attribute hesaplarının ve gameplay etkilerinin tamamı,
menu/inventory/effect state'i, reset konumunun tam Java spawn davranışı,
datapack custom dimension ve `client-parity` karşılaştırması açık kalır.
