# P08.ai — [server-conformance] Entity living flags semantic state

Bu kayıt, `ClientboundSetEntityDataPacket` içindeki Java `LivingEntity` living
flags alanının Rust `ClientWorld` state'ine semantik olarak uygulanmasını kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`LivingEntity.java:176-180`, bütün living entity'ler için ID `8` BYTE alanını
ve üç bit maskesini tanımlar: `0x01` item kullanımı, `0x02` kullanılan item'ın
off-hand'de olması ve `0x04` auto-spin attack. `LivingEntity.java:322-329`,
alanın başlangıç değerini `0` yapar.

`LivingEntity.java:3287-3289` auto-spin bitini; `3427-3433`
`isUsingItem()` ve kullanılan el sorgusunu okur. `3496-3505` bitleri OR/AND ile
değiştirir, `3507-3515` item kullanımı başlarken using ve hand bitlerini yazar,
`3624-3631` kullanım biterken using bitini temizler. `3273-3275` auto-spin
bitinin server-side temizlenmesini; `3523-3540` ise synced update sonrasındaki
item-use yan etkilerini gösterir. `Player.java:510-517` auto-spin başlatılırken
spin bitinin ayarlandığı player dalıdır.

`SynchedEntityData.java:74-85,108-125` metadata'yı yalnız gönderilen dirty
alanlarla günceller ve `ClientboundSetEntityDataPacket.java:21-37` ID,
serializer ve değer sırasını taşır. BYTE serializer ID `0`'dır
(`EntityDataSerializers.java:53-54,164-172`).

## Sözleşme

- Geçerli metadata içinde ID `8` ve BYTE serializer ID `0` varsa değer ham
  `u8` olarak saklanır ve `ClientEntity::living_flags` ile sorgulanır.
- `0x01` `is_using_item()`, `0x02` `used_item_is_off_hand()` ve `0x04`
  `is_auto_spin_attack()` sonuçlarını üretir. Diğer bitler korunur fakat bu
  dilimde semantik anlamlandırılmaz.
- Yeni genel entity ve local player living flags değerini Java synchronized
  default'u olan `0` ile başlatır. ID `8` içermeyen sonraki metadata packet'i
  önceki değeri korur; malformed veya eksik BYTE alanı state'i değiştirmez.
- Local player metadata'sındaki ID `8` aynı state'i günceller. Respawn'da
  `dataToKeep` bit `2` entity state'ini korur; bit yoksa yeni player flags değeri
  `0` olur.
- Java'nın item-use başlatma/durdurma, kullanılan item stack'i, hand animation,
  auto-spin hareketi ve synced-update yan etkileri bu metadata state taşıma
  diliminin dışındadır.

## Rust doğrulaması

`living_flags_from_metadata`, ID `8` BYTE alanını ortak metadata parser'ından
alır. `skip_metadata_value` BYTE serializer'ını tanıdığı için ID `8` öncesindeki
ve sonrasındaki bilinen alanlar birlikte gelen metadata içinde tüketilebilir.
`ClientEntity` ve `ClientPlayerState` ham byte'ı saklar; üç public getter Java
bit maskelerini aynı bit konumlarıyla sorgulatır.

World unit testi yeni entity'de default `0` değerini, üç bitin birlikte ve ayrı
olarak okunmasını ve yalnız başka bir metadata alanı geldiğinde living flags
değerinin korunmasını kontrol eder. Local player unit testi metadata
güncellemesini, `dataToKeep` bit `2` ile korunmasını ve bit yokken `0`'a
resetlenmesini kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken tracking alanı içinde `NoAI` ve `NoGravity` özellikli
bir iron golem oluşturuldu. Bu, vanilla'nın yeni living entity için ID `8`
default byte'ını metadata akışında göndermesini ve entity'nin probe süresince
konumunu korumasını sağladı:

```text
cargo run --locked --offline --example world_entity_living_flags_probe -- 127.0.0.1:25568 P08AiL1
summon minecraft:iron_golem 43 90 -36 {Health:75.0f,NoAI:1b,NoGravity:1b}
```

Vanilla sunucusu summon komutunu kabul etti. Probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `503` |
| `entity_living_flags` | `0` |
| `is_using_item` | `false` |
| `used_item_is_off_hand` | `false` |
| `is_auto_spin_attack` | `false` |
| `entity_type_id` | `70` |
| `entity_position` | `(43.5, 90.0, -35.5)` |
| `entity_metadata_length` | `13` |
| `world_entity_count` | `84` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `20205` |

Vanilla entity metadata akışındaki iron golem ID `8` BYTE değeri Rust
`ClientWorld` map'inde `0` olarak sorgulandı. Pozitif bit maskeleri Java dalı
unit testlerinde sentetik metadata ile doğrulandı; canlı server koşusu vanilla
varsayılan state'ini doğrular. Bu sonuç Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Item stack ve hand state'i, item-use lifecycle'ı, use animation, auto-spin
movement/pose, synced-update yan etkileri, diğer living metadata alanları ve
`client-parity` karşılaştırması açık kalır.
