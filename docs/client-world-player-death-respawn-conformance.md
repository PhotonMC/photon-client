# P08.i — [server-conformance] Aynı dimension ölüm respawn'ı ve `dataToKeep=0`

Bu kayıt, vanilla sunucunun ölen oyuncudan gelen `PERFORM_RESPAWN` komutunu
aynı dimension'da yeni local player state'ine dönüştürmesini kapsar. Canlı
kanıt sınıfı `server-conformance`'dır: Rust probe'u Mojang vanilla 26.2
dedicated server'ına bağlanır, sunucu konsolundan oyuncu öldürülür ve Rust
world kopyasındaki health, respawn ve local player state'i sorgulanır. Aynı
girdi Java istemcisiyle karşılaştırılmadığı için bu kayıt `client-parity`
değildir.

## Java kaynak incelemesi

`ClientboundSetHealthPacket.java:8-54`, health değerini big-endian float,
food değerini VarInt ve saturation değerini float olarak taşır.
`ServerboundClientCommandPacket.java:8-43`, `PERFORM_RESPAWN` enum değerini
ordinal `0` olarak yazar.

`ServerGamePacketListenerImpl.java:1891-1915`, `PERFORM_RESPAWN` komutunda
oyuncu health değeri pozitif değilse `PlayerList.respawn` çağrısını
`keepAllPlayerData=false` ve `KILLED` removal reason ile yapar. Oyuncu hâlâ
alive ise aynı komut sessizce reddedilir.

`PlayerList.java:392-431`, yeni `ServerPlayer` oluşturur, eski oyuncudan
`restoreFrom(..., false)` ile gerekli kalıcı state'i alır, aynı entity ID'yi
verir ve `dataToKeep` byte'ını `0` olarak clientbound respawn packet'ine koyar.
Respawn sonrasında yeni konum için ayrıca teleport correction ve health
packet'leri gönderebilir.

`ClientPacketListener.java:1247-1337`, dimension key aynıysa mevcut
`ClientLevel`'ı koruyup yeni `LocalPlayer` oluşturduğunu, ID'yi koruduğunu ve
keep bit 2 kapalıysa oyuncuyu reset konumuna alıp yaw'ı `-180` yaptığını
gösterir. Keep bit 1 kapalı olduğunda attribute'lar base değerlerine döner.

## Sözleşme

- Probe, ilk dünya yüklemesi ve `PLAYER_LOADED` sonrasında dışarıdan ölüm
  tetiklenmesini bekler; ölüm koşulu `ClientboundSetHealth` içindeki health
  değerinin `0` olmasıdır.
- Health `0` gözlenince Rust tam olarak bir `ServerboundClientCommand` paketi
  gönderir. Paket ID'si `12`, payload'ı `PERFORM_RESPAWN` için tek `0`
  baytıdır.
- Aynı dimension'daki ölüm respawn'ında world dimension key'i ve mevcut
  chunk/entity storage korunur; respawn packet'inin `dataToKeep` değeri `0`
  olmalıdır.
- Local player aynı ID ile yeni nesil olarak kurulmalı, bit 2 kapalı olduğu
  için önceki motion, rotation ve ham entity-data state'ini taşımamalıdır.
  Respawn sonrasında gelen player-position veya metadata packet'leri bu
  başlangıç state'ini güncelleyebilir.
- Bu dilim attribute değerlerinin, menu/inventory/effect state'inin ve Java
  client ekran/level-loading akışının tamamını kapatmaz.

## Rust doğrulaması

`PLAY_CLIENTBOUND_SET_HEALTH_PACKET_ID=104` ve
`PLAY_SERVERBOUND_CLIENT_COMMAND_PACKET_ID=12` Java 26.2 dispatch sırasından
alınmıştır. `parse_set_health_packet` üç health alanını packet sonuna kadar
tüketir; `encode_perform_respawn` enum ordinalını üretir.

`connect_and_world_player_death_respawn_probe`, health `0` görüldüğünde komutu
gönderir ve respawn packet'ini mevcut `ClientWorld` state'ine uygular. Respawn
gözlemi `dimension_changed`, `data_to_keep`, local player ID/nesli ve keep
bitlerini kaydeder. `ClientWorld::recreate_local_player`, bit 2 kapalı dalda
motion/metadata'yı temizler ve Java reset yaw'ı olan `-180` değerini başlangıç
state'ine yazar; sonraki correction packet'i nihai konumu/rotasyonu
güncelleyebilir.

Unit testler packet ID'lerini, health alan sırasını, trailing byte reddini ve
`PERFORM_RESPAWN` payload'ını doğrular. World testleri maskesiz respawn'da
nesil artışını, motion/rotation/entity-data temizliğini ve reset yaw'ını
kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Rust probe:

```text
cargo run --locked --offline --example world_player_death_respawn_probe -- 127.0.0.1:25568 <probe-user>
```

Probe dünya yüklemesini beklerken vanilla server konsolunda şu komut
çalıştırıldı:

```text
execute as <probe-user> run kill @s
```

Üç bağımsız vanilla oturumunda gözlenen sonuçlar:

| Gözlem | `P08iA` | `P08iB` | `P08iC` |
| --- | ---: | ---: | ---: |
| login player ID | `2404` | `2550` | `2702` |
| health observation count | `3` | `3` | `3` |
| zero health observed | `true` | `true` | `true` |
| respawn command packets sent | `1` | `1` | `1` |
| respawn packet count | `1` | `1` | `1` |
| respawn dimension | `minecraft:overworld` | `minecraft:overworld` | `minecraft:overworld` |
| `data_to_keep` | `0` | `0` | `0` |
| `keep_entity_data` | `false` | `false` | `false` |
| `keep_attribute_modifiers` | `false` | `false` | `false` |
| local player ID after respawn | `2404` | `2550` | `2702` |
| local player recreated | `true` | `true` | `true` |
| local player generation | `1` | `1` | `1` |
| dimension changed | `false` | `false` | `false` |
| post-correction motion | `(0.0, 0.0, 0.0)` | `(0.0, 0.0, 0.0)` | `(0.0, 0.0, 0.0)` |

Üç koşuda da vanilla, ölüm health packet'inden sonra gönderilen tek
`PERFORM_RESPAWN` komutunu kabul etti. Respawn aynı `minecraft:overworld`
key'inde geldi; Rust `ClientWorld` dimension replacement sonucunu `false`
olarak kaydetti ve local player ID'sini koruyarak nesli `0`'dan `1`'e çıkardı.
İlk respawn state'i sonraki server correction packet'leriyle güncellendi;
bu yüzden tabloda correction sonrası motion verisi yer alır.

Bu sonuç vanilla sunucusunun Rust paketini kabul ettiğini ve Rust world
state'inin bu akışa bağlandığını gösterir; Java client eşitliği iddiası
değildir.

## Açık kapsam

`dataToKeep` bit 1'in gerçek attribute değerlerine etkisi, death screen ve
level-loading UI state'i, menu/inventory/effect state'i, respawn config ve
anchor dalları, datapack custom dimension, exact pre-correction spawn state'i
ve `client-parity` karşılaştırması açık kalır.
