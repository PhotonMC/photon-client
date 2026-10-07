# P08.al.1 — ClientWorld entity effect ambience — [client-parity]

Bu kayıt, Java `LivingEntity` metadata içindeki ID `11` effect ambience
BOOLEAN alanının Rust `ClientWorld` kopyasına uygulanmasını ve aynı sentetik
metadata dizisinde Java ile eşleşmesini kapsar. Kanıt sınıfı `client-parity`'dir:
Java 26.2 living entity fixture'ı ile production Rust adapter aynı girdiyi alır.

Önceki [P08.al server-conformance kaydı](client-world-entity-effect-ambience-conformance.md)
Mojang vanilla 26.2 dedicated server'ın bu metadata akışını kabul ettiğini
gösterir. Bu kayıt canlı sunucu wire uyumluluğundan ayrı olarak istemci
world-copy state'inin Java ile eşitliğini ölçer.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `world/entity/LivingEntity.java:176-186,322-329`, effect ambience alanının
  ID `11`, `BOOLEAN` serializer ve `false` başlangıç değeriyle tanımlandığını
  gösterir.
- `world/entity/LivingEntity.java:870-874`, client tick'inde ambience değerini
  yalnız dolu effect particle listesi için particle sıklığı kararında okur.
- `world/entity/LivingEntity.java:901-909,960-970`, görünür effect'lerin tümü
  ambient olduğunda ID `11` değerinin `true`, görünür non-ambient effect
  olduğunda `false` yapıldığını tanımlar.
- `network/syncher/EntityDataSerializers.java:53-54,89-92,164-172` ve
  `network/codec/ByteBufCodecs.java:54-66,446-455`, BOOLEAN serializer ID'sinin
  `8`, boş particle listesi serializer'ının ID `17` ve list count'un VarInt
  olduğunu doğrular.
- `network/syncher/SynchedEntityData.java:74-85,108-125` ve
  `network/protocol/game/ClientboundSetEntityDataPacket.java:21-37`, yalnız
  gönderilen alanların atanmasını ve `255` metadata sonlandırıcısını tanımlar.

Java oracle'ı gerçek `LivingEntity` data accessor'larını kuran `ArmorStand`
tabanlı özgün fixture kullanır. ID `11` accessor'ı doğrudan fixture'ın
`SynchedEntityData` state'inden okunur; particle listesi boşken sonraki Boolean
alanının da Java tarafından tüketildiği ayrıca yakalanır.

## Yeni dal ve varyant kararı

Bu dilim, önceki P08.al server-conformance kaydındaki Java ID `11` BOOLEAN
assignment dalını ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem,
ambience değerinin `false` başlangıcı, `true` kurulumu, living flag ile birlikte
partial update, boş particle listesi sonrasında korunması ve temizlenmesidir.
Aynı sonucu veren mevcut `client-parity` kayıtlarının sayısı **0**'dır;
P08.z.1, P08.aa.1, P08.ad.1, P08.ae.1, P08.ag.1, P08.ai.1, P08.ah.1, P08.aj.1
ve P08.ak.1 farklı metadata alanlarını hedefler.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Living fixture'ı
  `living_flags=0` ve `effect_ambience=false` ile başlar.
- `0b0801ff`, ID `11`, BOOLEAN serializer ID `8` ve `true` değerini taşır.
- `0800010b0801ff`, living flags `0x01` ile ambience `true` değerini aynı
  metadata dizisinde taşır; iki alan da uygulanır.
- `0a11000b0801ff`, boş ID `10` particle listesi sonrasında ID `11` ambience
  değerini taşır. Boş liste sonraki metadata alanını kaydırmaz.
- `050801ff` yalnız no-gravity alanını taşır; ambience ve living flags önceki
  değerlerini korur.
- `0b0800ff` ambience değerini `false` yapar; living flags `1` olarak kalır.
- Dolu particle option codec'leri, particle sıklığı/seed/render'ı, effect
  lifecycle ve local player parity'si bu entity state kaydının dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-effect-ambience` kaydı
Java oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`bed53583426576dc337ac1568e801f2bcafa7ae456f72c33cd72e132ec1d8bfe`

Math corpus'u **2986** kayıttır; fark `null` kalmıştır. Referansın gözlenen
durumları şöyledir:

| Adım | `living_flags` | `effect_ambience` |
| --- | ---: | --- |
| initial | 0 | false |
| set-effect-ambience | 0 | true |
| combined-living-and-effect-ambience | 1 | true |
| empty-particles-preserves-effect-ambience | 1 | true |
| no-gravity-preserves-effect-ambience | 1 | true |
| clear-effect-ambience | 1 | false |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak ID `11` BOOLEAN
metadata'sını çözer. `ClientEntity::is_effect_ambient` state'i world
kopyasından sorgulanabilir; adapter Java ile aynı partial metadata sırasını
uygular.

Debug adapter SHA-256 değeri
`d8d28edecff41472b5996363c63790e1c8efc64da5c179a896209ee5bf8f9269` ile aynı
capture üzerinde **2986/2986** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/debug/examples/behavior_adapter
case_count=2986
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`dfce7ab05b63496193422384a623fe51fd54e703764c4e0332569bf9af99aa66` ile aynı
capture üzerinde **2986/2986** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/release/examples/behavior_adapter
case_count=2986
difference=null
status=parity_passed
```

Bu dilim serverbound paket üretmediği için vanilla sunucusuna ek bir bağlanma
sonucu Java client-parity karşılaştırmasına yeni kanıt eklemez; canlı sonuç
[P08.al server-conformance kaydında](client-world-entity-effect-ambience-conformance.md)
tutulur.

## Açık kapsam

Dolu particle listesi ve option codec'leri, ambience'e göre particle sıklığı,
random/seed, particle render'ı, görünür effect listesi, effect lifecycle,
local player parity'si, diğer living serializer'ları ve oynanabilir istemci
bağlantısı açık kalır.
