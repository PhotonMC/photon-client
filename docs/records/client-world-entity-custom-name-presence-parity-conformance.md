# P08.ag.1 — ClientWorld entity custom-name presence — [client-parity]

Bu kayıt, Java entity metadata içindeki ID `2` `OPTIONAL_COMPONENT` alanının
Rust `ClientWorld` kopyasına uygulanmasını ve custom-name presence ile
visibility state'inin aynı sentetik metadata dizisinde Java ile eşleşmesini
kapsar. Kanıt sınıfı `client-parity`'dir: Java 26.2 entity fixture'ı ile
production Rust adapter aynı girdiyi alır.

Önceki [P08.ag server-conformance kaydı](client-world-entity-custom-name-presence-conformance.md)
Mojang vanilla 26.2 dedicated server'ın custom-name metadata akışını kabul
ettiğini gösterir. Bu kayıt canlı sunucu wire uyumluluğundan ayrı olarak
istemci state'inin Java ile eşitliğini ölçer.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `world/entity/Entity.java:264-266,315-317,3293-3304`, ID `2` alanının
  `Optional<Component>` olduğunu, yeni entity'de boş başladığını ve
  `hasCustomName()` sonucunun Optional presence'ına bağlı olduğunu gösterir.
- `world/entity/Entity.java:3355-3369`, temel entity için `shouldShowName()`
  sonucunun ayrı custom-name visibility alanından geldiğini gösterir.
- `network/syncher/EntityDataSerializers.java:53-54,164-172` ve
  `network/codec/ByteBufCodecs.java:373-387`, `OPTIONAL_COMPONENT` serializer
  ID'sinin `6` olduğunu, presence byte'ını ve mevcut component'in NBT root'unu
  taşıdığını doğrular.
- `network/protocol/game/ClientboundSetEntityDataPacket.java:21-37` ve
  `network/syncher/SynchedEntityData.java:108-125`, metadata sonlandırıcısını
  ve yalnız gönderilen data item'larının atanmasını tanımlar.
- `network/chat/ComponentSerialization.java:36-41,89-120`, trusted component
  codec'inin NBT değerinden component çözdüğünü gösterir.

## Yeni dal ve varyant kararı

Bu dilim, önceki P08.ag server-conformance kaydındaki Java ID `2`
`OPTIONAL_COMPONENT` metadata dalını ilk kez `client-parity` sınıfında
çalıştırır. Farklı gözlem, mevcut component payload'ı ile presence'ın true
olması, yalnız visibility güncellemesinin bu state'i koruması, optional alanın
temizlenmesi ve component'in visibility false iken yeniden kurulmasıdır. Aynı
sonucu veren mevcut `client-parity` kayıtlarının sayısı **0**'dır; P08.z.1,
P08.aa.1, P08.ad.1 ve P08.ae.1 farklı metadata alanlarını hedefler.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Entity fixture'ı
  yeni entity olarak `custom_name_present=false` ve
  `custom_name_visible=false` ile başlar.
- `02060108000c446973706c6179206e616d65ff`, ID `2`, serializer `6`, presence
  byte'ı `1` ve geçerli NBT string component'i taşır. Sonuç presence `true`,
  visibility `false`, base entity `should_show_name=false` olur.
- `030801ff` yalnız ID `3` visibility alanını true yapar. Component presence
  korunur ve `should_show_name` true olur.
- `020600ff` optional component'i temizler; visibility true kalır. Java base
  entity'nin `shouldShowName()` sonucu visibility alanına bağlı olduğu için
  `should_show_name` true kalır.
- Son metadata, component'i yeniden kurarken ID `3` visibility alanını false
  yapar. Presence true, visibility false ve `should_show_name=false` sonucu
  gözlenir.
- Component'in metin/style alanları bu state sözleşmesinde tutulmaz; payload'ın
  geçerli component olarak tüketilmesi ve presence'a dönüşmesi ölçülür.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-custom-name` kaydı Java
oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`0d696d93f17213641aca37d1a2d3ca6f04a5f78906c9ef30e55fc3e08e18ac14`

Math corpus'u **2981** kayıttır; fark `null` kalmıştır. Referansın gözlenen
durumları şöyledir:

| Adım | `custom_name_present` | `custom_name_visible` | `should_show_name` |
| --- | --- | --- | --- |
| initial | false | false | false |
| set-custom-name | true | false | false |
| set-visible-only | true | true | true |
| clear-custom-name | false | true | true |
| restore-hidden-custom-name | true | false | false |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak ID `2` optional
component metadata'sını çözer. `custom_name_present_from_metadata`, presence
byte'ını ve mevcut NBT root'unu tüketerek sonraki metadata alanının başlangıcını
korur. `ClientEntity::has_custom_name`, `is_custom_name_visible` ve
`should_show_name` state'leri world kopyasından sorgulanabilir; adapter Java ile
aynı partial update sırasını uygular.

Debug adapter SHA-256 değeri
`d16b5b1bb092cd899cd143ff699e2176e473eb1ff91a68dfa57835107fc24ed8` ile aynı
capture üzerinde **2981/2981** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/debug/examples/behavior_adapter
case_count=2981
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`0632897b5cf1bf957f0b5d6f618303fe22bdb2ec8fa74646dedf326aa1f324bf` ile aynı
capture üzerinde **2981/2981** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/release/examples/behavior_adapter
case_count=2981
difference=null
status=parity_passed
```

Vanilla 26.2 server üzerindeki önceki P08.ag probe'u
[server-conformance kaydında](client-world-entity-custom-name-presence-conformance.md)
tutulur. Bu dilim serverbound paket üretmediği için canlı vanilla sunucusuna
ek bir bağlanma sonucu Java client-parity karşılaştırmasına yeni kanıt eklemez.

## Açık kapsam

Custom component'in metin/style state'inin world kopyasında tutulması, entity
name çözümü, MobRenderer uzaklık/crosshair seçimi ve gerçek name label render'ı,
Player görünürlük override'ının renderer'a bağlanması, diğer serializer'lar,
entity type doğrulaması ve client tick/lifecycle entegrasyonu açık kalır.
