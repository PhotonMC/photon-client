# P08.am.1 — ClientWorld entity effect particle type list — [client-parity]

Bu kayıt, Java `LivingEntity` metadata içindeki ID `10` `PARTICLES` alanının
Rust `ClientWorld` kopyasına uygulanmasını ve desteklenen `ENTITY_EFFECT`
option'larının aynı sentetik metadata dizisinde Java ile eşleşmesini kapsar.
Kanıt sınıfı `client-parity`'dir: Java 26.2 living entity fixture'ı ile
production Rust adapter aynı girdiyi alır.

Önceki [P08.am server-conformance kaydı](client-world-entity-effect-particles-conformance.md)
Mojang vanilla 26.2 dedicated server'ın bu metadata akışını kabul ettiğini
gösterir. Bu kayıt canlı sunucu wire uyumluluğundan ayrı olarak istemci
world-copy state'inin Java ile eşitliğini ölçer.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `world/entity/LivingEntity.java:176-186,322-329,901-909`, effect particle
  listesinin ID `10`, `PARTICLES` serializer ve boş başlangıçla tanımlandığını;
  görünür effect option'larının listeye sırayla yazıldığını gösterir.
- `network/syncher/EntityDataSerializers.java:89-92,164-172` ve
  `network/codec/ByteBufCodecs.java:446-455`, `PARTICLES` serializer ID'sinin
  `17` olduğunu, count'un VarInt taşındığını ve option codec'inin her element
  için çalıştığını doğrular.
- `core/particles/ParticleTypes.java:54-56,174-177`, `ENTITY_EFFECT`
  particle type'ının registry kaydını ve registry tabanlı stream codec'ini
  tanımlar. Pinned registration sırasındaki type ID `28`'dir.
- `core/particles/ColorParticleOption.java:18-20`, `ENTITY_EFFECT` option'ının
  type ID'den sonra ARGB rengini dört byte INT olarak taşıdığını gösterir.
- `network/syncher/SynchedEntityData.java:74-85,108-125` ve
  `network/protocol/game/ClientboundSetEntityDataPacket.java:21-37`, yalnız
  gönderilen alanların atanmasını ve `255` metadata sonlandırıcısını tanımlar.

Java oracle'ı gerçek `LivingEntity` data accessor'larını kuran `ArmorStand`
tabanlı özgün fixture kullanır. Particle registry lookup, metadata option'ı
Java'nın gerçek `PARTICLES` codec'iyle çözebilmesi için built-in registry
access üzerinden sağlanır; çıktı option renklerini değil registry type ID
listesini ölçer.

## Yeni dal ve varyant kararı

Bu dilim, önceki P08.am server-conformance kaydındaki Java ID `10` `PARTICLES`
assignment dalını ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem,
boş başlangıç, tek ve çift `ENTITY_EFFECT` option listesi, ARGB payload'ının
tüketilmesi, living flag ile birlikte partial update, liste korunması ve boş
listeyle temizlenmesidir. Aynı sonucu veren mevcut `client-parity` kayıtlarının
sayısı **0**'dır; P08.al.1 ambience Boolean alanını hedefler, P08.z.1,
P08.aa.1, P08.ad.1, P08.ae.1, P08.ag.1, P08.ai.1, P08.ah.1, P08.aj.1 ve
P08.ak.1 farklı metadata alanlarını hedefler.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Living fixture'ı
  `living_flags=0`, `effect_ambience=false` ve boş particle type listesiyle
  başlar.
- `0a11011c11223344ff`, ID `10`, `PARTICLES` serializer ID `17`, count `1`,
  `ENTITY_EFFECT` type ID `28` ve ARGB payload `11223344` taşır.
- `0800010a11021c112233441c55667788ff`, living flags `0x01` ile iki
  `ENTITY_EFFECT` option'ını aynı metadata dizisinde taşır; type listesi
  `[28,28]` olur.
- `050801ff` yalnız no-gravity alanını taşır; particle listesi ve living flags
  önceki değerlerini korur.
- `0a1100ff` geçerli boş liste değeridir; önceki particle type listesi temizlenir.
- Bu dilim ARGB renklerini saklamaz; diğer particle option codec'leri, particle
  sıklığı/seed/render'ı, effect lifecycle ve local player parity'si kapsam dışıdır.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-effect-particles` kaydı
Java oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`11c89a165809f040df3feeb21630586e7499f1438579c166936faab058f877f2`

Math corpus'u **2987** kayıttır; fark `null` kalmıştır. Referansın gözlenen
durumları şöyledir:

| Adım | `living_flags` | `effect_ambience` | `effect_particle_type_ids` |
| --- | ---: | --- | --- |
| initial | 0 | false | [] |
| set-one-effect-particle | 0 | false | [28] |
| set-two-effect-particles-with-living-flag | 1 | false | [28, 28] |
| no-gravity-preserves-effect-particles | 1 | false | [28, 28] |
| clear-effect-particles | 1 | false | [] |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak ID `10`
`PARTICLES` metadata'sını çözer. `read_effect_particle_type_ids`, count ve
`ENTITY_EFFECT` option'larının type ID ile ARGB payload'ını tüketir;
`ClientEntity::effect_particle_type_ids` yalnız type ID listesini world
kopyasından sorgular. Adapter Java ile aynı partial metadata sırasını uygular.

Debug adapter SHA-256 değeri
`7e960cd154418d18a5fc21b31bba9c2c27f9a23c85aa6176b0a127db184057b6` ile aynı
capture üzerinde **2987/2987** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/debug/examples/behavior_adapter
case_count=2987
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`3a92655687b59461df8c284ed87423e5057a0a2e38575ea3e57b3d6117af996a` ile aynı
capture üzerinde **2987/2987** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/release/examples/behavior_adapter
case_count=2987
difference=null
status=parity_passed
```

Bu dilim serverbound paket üretmediği için vanilla sunucusuna ek bir bağlanma
sonucu Java client-parity karşılaştırmasına yeni kanıt eklemez; canlı sonuç
[P08.am server-conformance kaydında](client-world-entity-effect-particles-conformance.md)
tutulur.

## Açık kapsam

Diğer particle type ve option codec'leri, ARGB renginin world state'ine alınması,
birden fazla effect listesinin lifecycle'ı, particle sıklığı/seed/renderer,
effect lifecycle, local player parity'si, diğer living serializer'ları ve
oynanabilir istemci bağlantısı açık kalır.
