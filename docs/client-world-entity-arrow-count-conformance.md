# P08.ak — [server-conformance] Entity living arrow count semantic state

Bu kayıt, `ClientboundSetEntityDataPacket` içindeki Java `LivingEntity` ok
sayısı alanının Rust `ClientWorld` state'ine uygulanmasını kapsar. Kanıt sınıfı
`server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2 dedicated
server'a bağlanır. Java istemcisiyle aynı girdide state karşılaştırılmadığı için
bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`LivingEntity.java:176-186`, yaşayan entity'ler için arrow count alanını ID `12`
ve `EntityDataSerializers.INT` ile tanımlar. `LivingEntity.java:322-329`, alanı
`0` ile başlatır; `LivingEntity.java:2000-2013`, getter/setter'ın synchronized
entity data değerini doğrudan okuduğunu ve yazdığını gösterir.

`EntityDataSerializers.java:53-54,164-172`, INT serializer'ının registration
sırasındaki ID'sini `1` olarak verir. `ByteBufCodecs.java:106-113` INT/VarInt
codec'ini tanımlar. `SynchedEntityData.java:74-85,108-125` yalnız değişen data
item'larının dirty metadata'ya girdiğini ve `ClientboundSetEntityDataPacket`
`21-37` metadata listesinin ID, serializer, değer ve `255` EOF sırasını
taşıdığını gösterir.

Ok çarpışmasındaki kaynak dalı `AbstractArrow.java:478-481` içindedir:
sunucu tarafında pierce seviyesi sıfır olan okun başarılı hasarından sonra
hedef `LivingEntity` ise arrow count bir artırılır. Client render tarafı
`AvatarRenderer.java:61-68,167-174` içinde `ArrowLayer`'ı ekler ve entity'nin
arrow count'unu render state'e taşır; `ArrowLayer.java:12-25` bu sayıyı stuck
ok adedi olarak kullanır. `LivingEntity.java:2769-2779` sunucu tick'inde zaman
dolunca sayının azaltılabildiğini de gösterir.

## Sözleşme

- Geçerli metadata içinde ID `12` ve INT serializer ID `1` varsa signed VarInt
  değer `i32` olarak okunur ve `ClientEntity::arrow_count` üzerinden sorgulanır.
- Yeni entity ve local player arrow count değeri `0` ile başlar. Yalnız ID `12`
  içeren metadata packet'i `[12, 1, value, 255]` biçiminde uygulanabilir.
- Metadata yalnız gönderilen alanları günceller. ID `12` içermeyen sonraki
  packet önceki sayıyı korur; yanlış serializer veya eksik değer de state'i
  değiştirmez.
- Local player metadata'sı aynı değeri player world kopyasına uygular.
  Respawn `KEEP_ENTITY_DATA` maskesiyle entity data korunduğunda arrow count
  korunur; boş keep maskesi yeni player varsayılanı olan `0` değerine döner.
- Bu dilim projectile hareketini, çarpışma geometrisini, hasar uygulamasını,
  ok modelini veya render katmanını uygulamaz; yalnız packet'ten sorgulanabilir
  living entity state'ini taşır.

## Rust doğrulaması

`arrow_count_from_metadata`, mevcut signed VarInt metadata okuyucusunu ID `12`
için kullanır. `ClientEntity` ve `ClientPlayerState` bu değeri ayrı bir `i32`
alanında tutar. Metadata uygulama yolu remote entity ve local player dallarında
değeri günceller; entity-data korumalı local player yeniden oluşturmasında
değer kopyalanır.

World unit testleri entity'de `0` başlangıcını, `3` değerini, kısmi metadata
sonrasında korunmasını ve yanlış serializer'ın reddedilmesini kontrol eder.
Local player testi metadata değerini, `KEEP_ENTITY_DATA` ile korunmasını ve
boş keep maskesiyle `0` değerine dönmesini kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken sabit konumda NoAI/no-gravity iron golem oluşturuldu.
Ardından gravity'siz bir vanilla arrow, hedefin arkasından hedefe doğru hareket
edecek şekilde summon edildi. Böylece count değeri NBT ile uydurulmadan vanilla
`AbstractArrow` çarpışma dalıyla üretildi:

```text
cargo run --locked --offline --example world_entity_arrow_count_probe -- 127.0.0.1:25568 P08ArrowL1
summon minecraft:iron_golem 55 90 -36 {NoAI:1b,NoGravity:1b,Health:75.0f}
summon minecraft:arrow 55.5 90 -45 {NoGravity:1b,Motion:[0.0d,0.0d,1.0d],damage:0.0d,pickup:0b}
```

Vanilla sunucusu iki summon komutunu kabul etti; probe hedefe gelen metadata
packet'ini uygulayıp başarıyla tamamlandı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `367` |
| `entity_arrow_count` | `1` |
| `entity_type_id` | `70` |
| `entity_position` | `(55.5, 90.0, -35.5)` |
| `entity_metadata_length` | `4` |
| `world_entity_count` | `67` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `16792` |

`entity_metadata_length=4`, hedefteki canlı update'in ID `12`, INT serializer,
VarInt `1` ve `255` EOF alanlarından oluştuğunu doğrular. Bu sonuç Java
istemcisiyle parity iddiası değildir.

## Açık kapsam

Stinger count ID `13`, arrow count'un tick ile azalması, projectile çarpışma ve
hasar fiziği, oyuncu/remote entity render katmanları, item/equipment state'i,
diğer living metadata alanları ve `client-parity` karşılaştırması açık kalır.
