# P08.ai.1 — ClientWorld entity living flags — [client-parity]

Bu kayıt, Java `LivingEntity` metadata içindeki ID `8` living flags byte'ının
Rust `ClientWorld` kopyasına uygulanmasını ve üç bit türevinin aynı sentetik
metadata dizisinde Java ile eşleşmesini kapsar. Kanıt sınıfı `client-parity`'dir:
Java 26.2 living entity fixture'ı ile production Rust adapter aynı girdiyi alır.

Önceki [P08.ai server-conformance kaydı](client-world-entity-living-flags-conformance.md)
Mojang vanilla 26.2 dedicated server'ın living flags metadata akışını kabul
ettiğini gösterir. Bu kayıt canlı sunucu wire uyumluluğundan ayrı olarak
istemci state'inin Java ile eşitliğini ölçer.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `world/entity/LivingEntity.java:176-180,322-329`, ID `8` alanının `BYTE`
  serializer ile `0` başladığını ve üç maskeyi tanımlar: item kullanımı `0x01`,
  kullanılan elin off-hand olması `0x02`, auto-spin attack `0x04`.
- `world/entity/LivingEntity.java:3273-3277,3425-3433`, auto-spin ve item-use
  sorgularının bu byte'tan türediğini gösterir.
- `network/syncher/EntityDataSerializers.java:53-54,164-172` ve
  `network/protocol/game/ClientboundSetEntityDataPacket.java:21-37`, BYTE
  serializer ID'sini `0` ve metadata sonlandırıcısını doğrular.
- `network/syncher/SynchedEntityData.java:108-125`, yalnız gönderilen data
  item'larının entity state'ine atanmasını tanımlar.

Java oracle'ı bu dal için `ArmorStand` tabanlı özgün bir living fixture
kullanır; fixture'ın client level'i diğer metadata oracle'larında kullanılan
sentetik level'dir. Bu seçim, living data accessor'larının constructor ile
kurulmasını ve Java `LivingEntity` getter'larının doğrudan sorgulanmasını
sağlar.

## Yeni dal ve varyant kararı

Bu dilim, önceki P08.ai server-conformance kaydındaki Java ID `8` living flags
assignment dalını ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem,
ham byte'ın `isUsingItem`, kullanılan el ve `isAutoSpinAttack` türevlerine
ayrılması, başka ID'lerle birlikte gelen packet'te korunması ve sıfırlanmasıdır.
Aynı sonucu veren mevcut `client-parity` kayıtlarının sayısı **0**'dır; P08.z.1,
P08.aa.1, P08.ad.1, P08.ae.1 ve P08.ag.1 farklı metadata alanlarını hedefler.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Living fixture'ı
  `living_flags=0` ile başlar.
- `080007ff`, ID `8`, BYTE serializer ID `0` ve `0x07` değerini taşır. Ham
  byte `7`, üç living flag getter'ının tamamını true yapar.
- `000040010107080001ff`, shared flags `0x40`, air supply `7` ve living flags
  `0x01` alanlarını aynı metadata'da taşır. Living state `1` olur; item-use
  true, off-hand false ve auto-spin false kalır.
- `080000ff` yalnız living flags byte'ını temizler; sonraki state `0` ve üç
  türevin tamamı false olur.
- ID `8` dışında gelen metadata alanları living flags değerini değiştirmez;
  item stack, hand lifecycle, animation, auto-spin movement ve synced-update
  yan etkileri bu kaydın dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-living-flags` kaydı Java
oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`f300783ca96867b79b1aebfc9e4206f7cb6631a869d0017de7f81cff2973eef0`

Math corpus'u **2982** kayıttır; fark `null` kalmıştır. Referansın gözlenen
durumları şöyledir:

| Adım | `living_flags` | `is_using_item` | `used_item_is_off_hand` | `is_auto_spin_attack` |
| --- | ---: | --- | --- | --- |
| initial | 0 | false | false | false |
| set-all-living-flags | 7 | true | true | true |
| combined-shared-air-and-living | 1 | true | false | false |
| clear-living-flags | 0 | false | false | false |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak ID `8` BYTE
metadata'sını çözer. `ClientEntity::living_flags` ve üç public getter Java
maskelerini aynı bit konumlarıyla sorgular; adapter metadata alanlarını Java
fixture'ıyla aynı sırada uygular.

Debug adapter SHA-256 değeri
`5ac2a3faa4affb790baf57294669dba37d3b9199affcd1b7b0fb3b174a957cef` ile aynı
capture üzerinde **2982/2982** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/debug/examples/behavior_adapter
case_count=2982
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`055596b3ff64e0c45d7f2be7b81abcc393a583f58b59726a726664634291ddbb` ile aynı
capture üzerinde **2982/2982** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/release/examples/behavior_adapter
case_count=2982
difference=null
status=parity_passed
```

Vanilla 26.2 server üzerindeki önceki P08.ai probe'u
[server-conformance kaydında](client-world-entity-living-flags-conformance.md)
tutulur. Bu dilim serverbound paket üretmediği için canlı vanilla sunucusuna
ek bir bağlanma sonucu Java client-parity karşılaştırmasına yeni kanıt eklemez.

## Açık kapsam

Item stack ve hand state'i, item-use lifecycle'ı, use animation, auto-spin
movement/pose, synced-update yan etkileri, health/effect/arrow/sleeping
metadata alanları, gerçek LivingEntity tick'i ve oynanabilir istemci bağlantısı
açık kalır.
