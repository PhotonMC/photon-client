# P08.ah.1 — ClientWorld entity health metadata — [client-parity]

Bu kayıt, Java `LivingEntity` metadata içindeki ID `9` health FLOAT alanının
Rust `ClientWorld` kopyasına uygulanmasını ve metadata geldikten sonraki health
state'inin aynı sentetik akışta Java ile eşleşmesini kapsar. Kanıt sınıfı
`client-parity`'dir: Java 26.2 living entity fixture'ı ile production Rust
adapter aynı girdiyi alır.

Önceki [P08.ah server-conformance kaydı](client-world-entity-health-conformance.md)
Mojang vanilla 26.2 dedicated server'ın health metadata akışını kabul ettiğini
gösterir. Bu kayıt canlı sunucu wire uyumluluğundan ayrı olarak istemci
state'inin Java ile eşitliğini ölçer.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `world/entity/LivingEntity.java:176-180,286-290,322-329`, ID `9` alanının
  `FLOAT` serializer ile tanımlandığını, synchronized default'un `1.0F`
  olduğunu ve constructor'ın entity türünün max-health değerini yazdığını
  gösterir.
- `world/entity/LivingEntity.java:1163-1168`, health getter'ının synchronized
  state'i döndürdüğünü; metadata assignment'ın gelen FLOAT değerini doğrudan
  state'e taşıdığını doğrular.
- `network/syncher/EntityDataSerializers.java:53-56,164-172` ve
  `network/codec/ByteBufCodecs.java:132-139`, FLOAT serializer ID'sinin `3`
  olduğunu ve dört byte IEEE-754 değer taşıdığını tanımlar.
- `network/syncher/SynchedEntityData.java:108-125` ve
  `network/protocol/game/ClientboundSetEntityDataPacket.java:21-37`, yalnız
  gönderilen metadata alanlarının atanmasını ve `255` sonlandırıcısını verir.

Java oracle'ı `ArmorStand` tabanlı living fixture kullanır; ID `9` accessor'ı
constructor ile kurulmuş gerçek `LivingEntity` data tablosundan okunur.

## Yeni dal ve varyant kararı

Bu dilim, önceki P08.ah server-conformance kaydındaki Java ID `9` FLOAT
assignment dalını ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem,
ID `8` öncesinden geçilerek health'in `7.5` yapılması, `0.0` değerinin geçerli
olarak saklanması, başka metadata ile korunması ve signed negatif FLOAT
bitlerinin taşınmasıdır. Aynı sonucu veren mevcut `client-parity` kayıtlarının
sayısı **0**'dır; P08.z.1, P08.aa.1, P08.ad.1, P08.ae.1, P08.ag.1 ve P08.ai.1
farklı state alanlarını hedefler.

Java living constructor'ı entity türünün max-health başlangıcını kurduğu, genel
Rust `ClientEntity` ise tür bağımsız olarak `health=None` ile başladığı için
başlangıç state'i bu karşılaştırmaya dahil edilmez. İlk ID `9` metadata'sından
sonra iki tarafın ortak gözlenebilir state'i karşılaştırılır; bu başlangıç
farkı gizlenmez ve tür bazlı max-health bağlama açık kapsamda kalır.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Her gözlem ilk
  health metadata'sı uygulandıktan sonra alınır.
- `080000090340f00000ff`, living flags `0` ve health `7.5` değerini taşır.
  Health f32 raw bitleri `40f00000` olur.
- `090300000000ff` health'i `0.0` yapar; raw bitler `00000000` olur.
- `050801ff` yalnız no-gravity metadata'sını taşır ve health `0.0` olarak
  korunur.
- `0903bf800000ff` health'i `-1.0` yapar; Java setter/clamp veya ölüm
  lifecycle'ı bu metadata assignment kaydında çağrılmaz, raw bitler
  `bf800000` olarak kalır.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-health` kaydı Java
oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`ee6ac8f6934b7683927059196b35a8cc37a5eac4c89da30cf229d2f838c23c6d`

Math corpus'u **2983** kayıttır; fark `null` kalmıştır. Referansın gözlenen
durumları şöyledir:

| Adım | `living_flags` | `health_bits` |
| --- | ---: | --- |
| set-health-after-living-flags | 0 | `40f00000` |
| set-zero-health | 0 | `00000000` |
| no-gravity-preserves-health | 0 | `00000000` |
| set-negative-health | 0 | `bf800000` |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak ID `9` FLOAT
metadata'sını çözer. `ClientEntity::health` raw f32 bitlerini korur; adapter
Java ile aynı metadata sırasını uygular ve ondalık yazım farkını önlemek için
health'i bit string olarak karşılaştırır.

Debug adapter SHA-256 değeri
`d097f957eee9a4de633fa86495e91d1f0fbee4c57cce14ae0327f5480f94e419` ile aynı
capture üzerinde **2983/2983** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/debug/examples/behavior_adapter
case_count=2983
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`fe91b1dc32dc20e2c7dd54bb48984eab6ae693b9f09cbf5af845904e3446a2e8` ile aynı
capture üzerinde **2983/2983** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/release/examples/behavior_adapter
case_count=2983
difference=null
status=parity_passed
```

Vanilla 26.2 server üzerindeki önceki P08.ah probe'u
[server-conformance kaydında](client-world-entity-health-conformance.md)
tutulur. Bu dilim serverbound paket üretmediği için canlı vanilla sunucusuna
ek bir bağlanma sonucu Java client-parity karşılaştırmasına yeni kanıt eklemez.

## Açık kapsam

Entity türlerinin max-health attribute çözümü, health başlangıcı ve setter
clamp'i, hasar/iyileşme/ölüm lifecycle'ı, local health HUD/renderer, health
tick davranışı, diğer living metadata alanları ve oynanabilir istemci bağlantısı
açık kalır.
