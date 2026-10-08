# P08.aj.1 — ClientWorld entity mob flags — [client-parity]

Bu kayıt, Java `Mob` metadata içindeki ID `15` mob flags byte'ının Rust
`ClientWorld` kopyasına uygulanmasını ve üç bit türevinin aynı sentetik metadata
dizisinde Java ile eşleşmesini kapsar. Kanıt sınıfı `client-parity`'dir: Java
26.2 mob fixture'ı ile production Rust adapter aynı girdiyi alır.

Önceki [P08.aj server-conformance kaydı](client-world-entity-mob-flags-conformance.md)
Mojang vanilla 26.2 dedicated server'ın mob flags metadata akışını kabul
ettiğini gösterir. Bu kayıt canlı sunucu wire uyumluluğundan ayrı olarak
istemci state'inin Java ile eşitliğini ölçer.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `world/entity/Mob.java:99-103,269-272`, ID `15` alanının `BYTE` serializer ile
  `0` başladığını ve NoAI `0x01`, left-handed `0x02`, aggressive `0x04`
  maskelerini tanımlar.
- `world/entity/Mob.java:1312-1340`, effective-AI ve üç public getter'ın bu
  byte'tan türediğini gösterir.
- `world/entity/Mob.java:1346-1348`, left-handed bitinin main-arm kararına
  aktarıldığını doğrular.
- `network/syncher/EntityDataSerializers.java:53-54,164-172` ve
  `network/protocol/game/ClientboundSetEntityDataPacket.java:21-37`, BYTE
  serializer ID'sini `0` ve metadata sonlandırıcısını verir.
- `network/syncher/SynchedEntityData.java:108-125`, yalnız gönderilen data
  item'larının atanmasını tanımlar.

Java oracle'ı `IronGolem` tabanlı özgün bir mob fixture kullanır. Fixture'ın
client level'i server goal registration dalını çalıştırmadan gerçek `Mob`
synched data tablosunu kurar; getter'lar doğrudan Java mob state'inden okunur.

## Yeni dal ve varyant kararı

Bu dilim, önceki P08.aj server-conformance kaydındaki Java ID `15` mob flags
assignment dalını ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem,
ham byte'ın NoAI, left-handed ve aggressive türevlerine ayrılması, başka base
metadata ile birlikte gelen packet'te korunması ve sıfırlanmasıdır. Aynı sonucu
veren mevcut `client-parity` kayıtlarının sayısı **0**'dır; P08.z.1, P08.aa.1,
P08.ad.1, P08.ae.1, P08.ag.1, P08.ai.1 ve P08.ah.1 farklı state alanlarını
hedefler.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Mob fixture'ı
  `mob_flags=0` ile başlar.
- `0f0007ff`, ID `15`, BYTE serializer ID `0` ve `0x07` değerini taşır. Ham
  byte `7`, üç mob flag getter'ının tamamını true yapar.
- `0000400101070f0002ff`, shared flags `0x40`, air supply `7` ve mob flags
  `0x02` alanlarını aynı metadata'da taşır. Mob state `2` olur; yalnız
  left-handed true kalır.
- `050801ff` yalnız no-gravity metadata'sını taşır ve mob flags `2` olarak
  korunur.
- `0f0000ff` yalnız mob flags byte'ını temizler; ham state `0` ve üç türevin
  tamamı false olur.
- NoAI'nin gerçek AI/tick yürütmesi, main-arm item state'i, combat/aggressive
  davranışı ve renderer/animation yan etkileri bu kaydın dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-mob-flags` kaydı Java
oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`73d767058d7bfb9170482f9a1a0d5bde24260711f985b69dda3c054202138e6e`

Math corpus'u **2984** kayıttır; fark `null` kalmıştır. Referansın gözlenen
durumları şöyledir:

| Adım | `mob_flags` | `is_no_ai` | `is_left_handed` | `is_aggressive` |
| --- | ---: | --- | --- | --- |
| initial | 0 | false | false | false |
| set-all-mob-flags | 7 | true | true | true |
| combined-shared-air-and-mob | 2 | false | true | false |
| no-gravity-preserves-mob | 2 | false | true | false |
| clear-mob-flags | 0 | false | false | false |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak ID `15` BYTE
metadata'sını çözer. `ClientEntity::mob_flags` ve üç public getter Java
maskelerini aynı bit konumlarıyla sorgular; adapter metadata alanlarını Java
fixture'ıyla aynı sırada uygular.

Debug adapter SHA-256 değeri
`3d18f347203f1bb28e6da2d58b33957a66ef18a6dca7225f2e5220d65884a239` ile aynı
capture üzerinde **2984/2984** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/debug/examples/behavior_adapter
case_count=2984
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`3d02147725fb558985d22d3ebbe0e6517cd97532ad420ddc841f9681802e8dbf` ile aynı
capture üzerinde **2984/2984** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/release/examples/behavior_adapter
case_count=2984
difference=null
status=parity_passed
```

Vanilla 26.2 server üzerindeki önceki P08.aj probe'u
[server-conformance kaydında](client-world-entity-mob-flags-conformance.md)
tutulur. Bu dilim serverbound paket üretmediği için canlı vanilla sunucusuna
ek bir bağlanma sonucu Java client-parity karşılaştırmasına yeni kanıt eklemez.

## Açık kapsam

Gerçek mob AI/tick yürütmesi, main-arm item/hand state'i, combat/aggressive
animasyonları, mob equipment metadata'sı, tür özel mob alanları, living
metadata yan etkileri ve oynanabilir istemci bağlantısı açık kalır.
