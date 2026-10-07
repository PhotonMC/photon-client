# P08.l — [server-conformance] LocalPlayer health, food ve saturation state'i

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte
tutar. Kapsam, clientbound `SetHealth` paketinin local player world state'ine
bağlanmasıdır. Canlı kanıt sınıfı `server-conformance`'dır: Rust probe'u
Mojang vanilla 26.2 dedicated server'ına bağlanır ve packet alanlarını
`ClientWorld` içindeki local player state'iyle birlikte sorgular. Aynı girdi
Java istemcisiyle karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundSetHealthPacket.java:8-54`, health float'ı, food VarInt'ini ve
saturation float'ını bu sırayla taşır. Dispatch kaydı
`GameProtocols.java:237` ve `GamePacketTypes.java:108` içindedir; bu sürümde
clientbound `SetHealth` packet ID'si `104`'tür.

`ClientPacketListener.java:1233-1237`, paketi doğrudan mevcut
`minecraft.player` üzerinde işler: health için `hurtTo`, food için
`FoodData.setFoodLevel` ve saturation için `FoodData.setSaturation` çağrılır.
`LivingEntity.java:1163-1169`, health state'inin entity data'da tutulduğunu ve
`setHealth` değerinin `0..maxHealth` aralığına sıkıştırıldığını gösterir.
`FoodData.java:14-15` yeni food state'i için `20` food ve `5.0F` saturation
varsayılanlarını tanımlar.

## Sözleşme

- `SetHealth` packet'i health float, food VarInt ve saturation float alanlarını
  packet sonuna kadar tüketir.
- Paket mevcut local player'a uygulanır; health, food ve saturation değerleri
  birlikte sorgulanabilir player state'ine yazılır. Packet gözlemindeki
  `applied_to_world` alanı bu uygulamayı kaydeder.
- Yeni local player nesnesi oluşturulduğunda Java constructor varsayılanları
  health `20.0`, food `20` ve saturation `5.0` ile başlar. Respawn sonrası
  sunucunun gönderdiği `SetHealth` paketi bu state'i günceller.
- Bu dilim health clamp işlemini, food exhaustion/natural regeneration tick'ini,
  death screen'i, absorption health'i ve gameplay etkilerini kapsamaz.

## Rust doğrulaması

`parse_set_health_packet`, Java alan sırasını ve packet sonunu doğrular.
`ClientPlayerState`, health ve saturation değerlerini ham float bitleriyle,
food değerini integer olarak tutar. `ClientWorld::apply_local_player_health`
yalnız mevcut local player state'ine yazar; player yeniden kurulurken
constructor varsayılanları geri gelir.

`world_player_death_respawn_probe` health `0` gözlemini ve bütün health
update'lerinin world'e uygulandığını kontrol eder; tamamlanan respawn sonrası
local player health/food/saturation değerlerini yazdırır. Unit test varsayılan
değerleri, death health update'ini ve yeni player ile reset'i doğrular.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

Rust probe dünya yüklemesini beklerken vanilla server konsolunda her oyuncu
aynı dimension'da öldürüldü:

```text
execute as <probe-user> run kill @s
```

Rust probe:

```text
cargo run --locked --offline --example world_player_death_respawn_probe -- 127.0.0.1:25568 <probe-user>
```

İki bağımsız geçerli vanilla oturumunda gözlenen sonuçlar:

| Gözlem | `P08lA` | `P08lB` |
| --- | ---: | ---: |
| login player ID | `3729` | `3868` |
| health observation count | `3` | `3` |
| zero health applied to world | `true` | `true` |
| all health updates applied to world | `true` | `true` |
| respawn command packets sent | `1` | `1` |
| respawn packet count | `1` | `1` |
| respawn dimension | `minecraft:overworld` | `minecraft:overworld` |
| `data_to_keep` | `0` | `0` |
| local player generation | `1` | `1` |
| final local player health | `20` | `20` |
| final local player food | `20` | `20` |
| final local player saturation | `5` | `5` |
| final world chunk count | `157` | `157` |

Her iki oturumda ölüm health packet'i `applied_to_world=true` ile local player
state'ine uygulandı, tek `PERFORM_RESPAWN` komutu kabul edildi ve aynı
dimension'da yeni player nesli oluşturuldu. Respawn sonrası sorgulanabilir
state vanilla varsayılanları olan `20/20/5` değerlerine döndü. Sonraki health
paketleri de local state'e uygulanmış olarak kaydedildi.

Bu sonuç vanilla sunucusunun Rust packet tüketimini ve local player health
state uygulamasını gösterir; Java client eşitliği iddiası değildir.

## Açık kapsam

Health clamp ve max-health attribute ilişkisi, food exhaustion ve natural
regeneration ticking'i, absorption, death screen/GUI, remote player health
state'i, menu/inventory state'i ve `client-parity` karşılaştırması açık kalır.
