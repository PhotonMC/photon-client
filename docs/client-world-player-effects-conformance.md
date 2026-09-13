# P08.k — [server-conformance] LocalPlayer mob-effect state'i

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte
tutar. Kapsam, clientbound mob-effect update/remove paketlerinin local player
world state'ine bağlanmasıdır. Canlı kanıt sınıfı `server-conformance`'dır:
Rust probe'u Mojang vanilla 26.2 dedicated server'ına bağlanır ve paket
alanlarını `ClientWorld` içindeki local player state'iyle birlikte sorgular.
Aynı girdinin Java istemcisiyle karşılaştırılması yapılmadığı için bu kayıt
`client-parity` değildir.

## Java kaynak incelemesi

`ClientboundUpdateMobEffectPacket.java:12-80`, entity ID'si ve mob-effect
registry holder ID'sinden sonra amplifier VarInt'i, duration VarInt'i ve flags
byte'ını taşır. Flag bitleri ambient `1`, visible `2`, show icon `4` ve blend
`8` olarak tanımlıdır. `ClientboundRemoveMobEffectPacket.java:12-35`, entity
ID'si ile effect registry ID'sini taşır. Dispatch kayıtları
`GameProtocols.java:211,265` ve `GamePacketTypes.java:82,131` içindedir; bu
sürümde packet ID'leri remove için `78`, update için `132`'dir.

`ClientPacketListener.java:1790-1810`, update paketinde hedef entity
`LivingEntity` ise packet alanlarıyla yeni `MobEffectInstance` oluşturur,
blend biti kapalıysa blending'i atlar ve aynı effect'i zorla ekler.
`ClientPacketListener.java:1976-1981`, remove paketini yalnızca mevcut
`LivingEntity` için `removeEffectNoUpdate` yoluna verir. Bilinmeyen veya living
olmayan hedeflerde bu yollar world effect state'i değiştirmez.

`ClientPacketListener.java:1247-1337`, respawn sırasında yeni `LocalPlayer`
nesnesinin oluşturulduğunu ve ID'nin eski oyuncudan aktarıldığını gösterir.
Bu kod yolunda eski effect listesini yeni oyuncuya kopyalayan bir çağrı yoktur;
effect'ler yeni oyuncuya sonraki update paketleriyle gelir.

## Sözleşme

- Update packet'i entity ID, effect registry ID, amplifier, duration ve flags
  alanlarını packet sonuna kadar tüketir. Effect registry ID'si ve flags ham
  numeric değer olarak korunur.
- Paket entity ID'si local player ID'siyle eşleştiğinde effect state'e yazılır.
  Aynı effect registry ID'si tekrar gelirse mevcut kayıt değiştirilir; yeni
  effect ID'si ayrı kayıt olarak eklenir.
- Remove packet'i local player ID'siyle eşleştiğinde ilgili effect kaydını
  siler. Bilinmeyen entity ID'leri gözlem olarak kalır ve local player state'ini
  değiştirmez.
- Respawn yeni local player nesnesi oluşturduğunda effect state'i yeni nesilde
  boş başlar; sonraki packet'ler bu listeyi doldurur.
- Bu dilim duration tick azaltımını, effect süresi dolma davranışını, effect
  registry ID'sinin isim çözümlemesini, render ikonlarını, blending'i ve effect
  gameplay etkilerini kapsamaz.

## Rust doğrulaması

`parse_update_mob_effect_packet` ve `parse_remove_mob_effect_packet`, Java
alan sırasını ve packet sonunu doğrular. `ClientMobEffectState`, effect ID,
amplifier, duration ve flags değerlerini sorgulanabilir state olarak tutar.
Packet dispatch'i yalnız local player ID'siyle eşleşen update/remove kaydını
`ClientWorld`'e uygular; her observation içindeki `applied_to_world` alanı bu
eşleşmeyi kaydeder.

`ClientWorld::apply_local_player_mob_effect` aynı effect'i değiştirerek
uygular, `remove_local_player_mob_effect` siler ve
`recreate_local_player` yeni neslin effect listesini boş başlatır. Unit test;
alanların sorgulanmasını, aynı ID'nin değiştirilmesini, silinmesini, bilinmeyen
oyuncu ID'sinin reddedilmesini ve respawn temizliğini doğrular.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
başlangıç oyun modu creative, `view-distance=5`, `simulation-distance=5`,
`entity-broadcast-range-percentage=100`, `127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Probe bağlandıktan sonra vanilla server konsolunda her oturum için oyuncu
survival'a alındı ve effect/item hazırlığı yapıldı:

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

| Gözlem | `P08kA` | `P08kB` |
| --- | ---: | ---: |
| login player ID | `3476` | `3605` |
| carried item packets sent | `1` | `1` |
| use item packets sent | `1` | `1` |
| local effect update | `entity=3476, effect=18, amplifier=0, duration=1200, flags=8, applied=true` | `entity=3605, effect=18, amplifier=0, duration=1200, flags=8, applied=true` |
| local effect removal | `entity=3476, effect=18, applied=true` | `entity=3605, effect=18, applied=true` |
| final local player effect count | `0` | `0` |
| preparation item | `count=1, item=1412, components=[17]` | `count=1, item=1412, components=[17]` |
| remainder item | `state=3, slot=36, count=1, item=1149, components=[]` | `state=3, slot=36, count=1, item=1149, components=[]` |
| block changed ACK sequences | `[29]` | `[29]` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet count | `4488` | `4517` |

Her iki oturumda local player effect update'i `applied_to_world=true` ile
state'e yazıldı. 40 tick'lik honey bottle tüketimi sonrasında aynı poison
effect'i için remove paketi yine local state'e uygulandı ve final liste boş
kaldı. `use_item_on` gönderilmedi; sequence `29` için ACK alındı. Item
component'li hazırlık stack'i ve glass bottle remainder'ı da iki oturumda
aynı gözlendi.

Bu sonuç vanilla sunucusunun Rust packet tüketimini ve local player effect
state uygulamasını gösterir; Java client eşitliği iddiası değildir.

## Açık kapsam

Effect duration ticking ve sona erme, effect registry isimleri, blending ve
ikon görünümü, attribute/effect gameplay etkileri, remote living entity effect
state'i, menu/inventory state'i ve `client-parity` karşılaştırması açık kalır.
