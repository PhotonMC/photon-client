# P08.ak.1 — ClientWorld entity living arrow count — [client-parity]

Bu kayıt, Java `LivingEntity` metadata içindeki ID `12` arrow count INT
alanının Rust `ClientWorld` kopyasına uygulanmasını ve signed VarInt state'inin
aynı sentetik metadata dizisinde Java ile eşleşmesini kapsar. Kanıt sınıfı
`client-parity`'dir: Java 26.2 living entity fixture'ı ile production Rust
adapter aynı girdiyi alır.

Önceki [P08.ak server-conformance kaydı](client-world-entity-arrow-count-conformance.md)
Mojang vanilla 26.2 dedicated server'ın ok sayısı metadata akışını kabul
ettiğini gösterir. Bu kayıt canlı sunucu wire uyumluluğundan ayrı olarak
istemci state'inin Java ile eşitliğini ölçer.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `world/entity/LivingEntity.java:176-186,322-329`, ID `12` alanının `INT`
  serializer ile `0` başladığını tanımlar.
- `world/entity/LivingEntity.java:2000-2005`, `getArrowCount()` ve setter'ın
  synchronized entity data değerini doğrudan kullandığını gösterir.
- `network/syncher/EntityDataSerializers.java:53-54,164-172` ve
  `network/codec/ByteBufCodecs.java:106-113`, INT serializer ID'sini `1` ve
  signed VarInt wire biçimini doğrular.
- `network/syncher/SynchedEntityData.java:108-125` ve
  `network/protocol/game/ClientboundSetEntityDataPacket.java:21-37`, yalnız
  gönderilen data item'larının atanmasını ve `255` sonlandırıcısını tanımlar.

Java oracle'ı gerçek `LivingEntity` data accessors'ını kuran `ArmorStand`
tabanlı özgün fixture kullanır; arrow count getter'ı doğrudan bu state'ten
okunur.

## Yeni dal ve varyant kararı

Bu dilim, önceki P08.ak server-conformance kaydındaki Java ID `12` INT
assignment dalını ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem,
pozitif count, signed negatif VarInt, başka metadata ile partial koruma ve
temizleme adımlarının Java ve Rust'ta aynı state'i üretmesidir. Aynı sonucu
veren mevcut `client-parity` kayıtlarının sayısı **0**'dır; P08.z.1, P08.aa.1,
P08.ad.1, P08.ae.1, P08.ag.1, P08.ai.1, P08.ah.1 ve P08.aj.1 farklı metadata
alanlarını hedefler.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Living fixture'ı
  `arrow_count=0` ve `living_flags=0` ile başlar.
- `0c0107ff`, ID `12`, INT serializer ID `1` ve `7` değerini taşır.
- `0800010c01ffffffff0fff`, living flags `0x01` ile arrow count `-1` değerini
  aynı metadata'da taşır. Signed VarInt'in beş byte biçimi korunur.
- `050801ff` yalnız no-gravity metadata'sını taşır; arrow count `-1`, living
  flags `1` olarak kalır.
- `0c0100ff` arrow count'u `0` yapar; living flags `1` olarak korunur.
- Projectile collision/hasar, count'un tick ile azalması, ok modeli ve render
  katmanı bu metadata state kaydının dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-arrow-count` kaydı Java
oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`3c458b2dd3fe65bfda06c4e486307fcddef1ea0431e14089bba96b2cc00e2746`

Math corpus'u **2985** kayıttır; fark `null` kalmıştır. Referansın gözlenen
durumları şöyledir:

| Adım | `arrow_count` | `living_flags` |
| --- | ---: | ---: |
| initial | 0 | 0 |
| set-positive-arrow-count | 7 | 0 |
| combined-living-and-negative-arrow-count | -1 | 1 |
| no-gravity-preserves-arrow-count | -1 | 1 |
| clear-arrow-count | 0 | 1 |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak ID `12` INT
metadata'sını çözer. `ClientEntity::arrow_count` signed `i32` state'ini
sorgular; adapter Java ile aynı metadata sırasını uygular.

Debug adapter SHA-256 değeri
`fbcb1130bf59cc3b56626563c33725f6051a862039efcc226d655e38f622d95f` ile aynı
capture üzerinde **2985/2985** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/debug/examples/behavior_adapter
case_count=2985
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`99ffedff788b26cbb4b1ff2c0be6cf25b8aae131c9d7c2d05a06aa8aa7ce9363` ile aynı
capture üzerinde **2985/2985** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/release/examples/behavior_adapter
case_count=2985
difference=null
status=parity_passed
```

Vanilla 26.2 server üzerindeki önceki P08.ak probe'u
[server-conformance kaydında](client-world-entity-arrow-count-conformance.md)
tutulur. Bu dilim serverbound paket üretmediği için canlı vanilla sunucusuna
ek bir bağlanma sonucu Java client-parity karşılaştırmasına yeni kanıt eklemez.

## Açık kapsam

Stinger count ID `13`, arrow count'un tick ile azalması, projectile çarpışma ve
hasar fiziği, oyuncu/remote entity render katmanları, item/equipment state'i,
diğer living metadata alanları ve oynanabilir istemci bağlantısı açık kalır.
