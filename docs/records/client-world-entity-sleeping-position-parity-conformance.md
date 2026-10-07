# P08.an.1 — ClientWorld entity sleeping position — [client-parity]

Bu kayıt, Java `LivingEntity` metadata içindeki ID `14` `OPTIONAL_BLOCK_POS`
alanının Rust `ClientWorld` kopyasına uygulanmasını ve Optional BlockPos
state'inin aynı sentetik metadata dizisinde Java ile eşleşmesini kapsar. Kanıt
sınıfı `client-parity`'dir: Java 26.2 living entity fixture'ı ile production
Rust adapter aynı girdiyi alır.

Önceki [P08.an server-conformance kaydı](client-world-entity-sleeping-position-conformance.md)
Mojang vanilla 26.2 dedicated server'ın bu metadata akışını kabul ettiğini
gösterir. Bu kayıt canlı sunucu wire uyumluluğundan ayrı olarak istemci
world-copy state'inin Java ile eşitliğini ölçer.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `world/entity/LivingEntity.java:187-189,322-330,3764-3777`, sleeping
  position alanının ID `14`, `OPTIONAL_BLOCK_POS` serializer ve
  `Optional.empty()` başlangıcıyla tanımlandığını; getter, temizleme ve
  `isSleeping()` sonuçlarının bu synchronized state'e bağlı olduğunu gösterir.
- `world/entity/LivingEntity.java:3523-3529`, client tarafında ID `14`
  değişince dolu sleeping position'ın `setPosToBed` yan etkisini tanımlar.
- `world/entity/LivingEntity.java:3780-3799`, gerçek uyku başlangıcında pose,
  yatak konumu, sleeping position ve hareket state'lerinin birlikte
  güncellendiğini gösterir. Bu kayıt yalnız metadata Optional state'ini ölçer.
- `network/syncher/EntityDataSerializers.java:94-96,173-175`,
  `OPTIONAL_BLOCK_POS` serializer ID'sinin `11` olduğunu ve
  `BlockPos.STREAM_CODEC` üzerine kurulduğunu doğrular.
- `network/codec/ByteBufCodecs.java:373-385`, Optional presence byte'ını ve
  değer varsa iç codec'i; `network/FriendlyByteBuf.java:385-400` ile
  `core/BlockPos.java:38-44,74-115`, packed position'ın sekiz byte big-endian
  long olarak taşınmasını tanımlar.
- `network/syncher/SynchedEntityData.java:74-85,108-125` ve
  `network/protocol/game/ClientboundSetEntityDataPacket.java:21-37`, yalnız
  gönderilen alanların atanmasını ve `255` metadata sonlandırıcısını tanımlar.

Java oracle'ı gerçek `LivingEntity` getter'larını kullanan `ArmorStand` tabanlı
özgün fixture çalıştırır. Boş Optional için Java Gson çıktısında alanın
serileştirilmemesi de gözlenen çıktı şeklinin parçası olarak korunur.

## Yeni dal ve varyant kararı

Bu dilim, önceki P08.an server-conformance kaydındaki Java ID `14`
`OPTIONAL_BLOCK_POS` assignment dalını ilk kez `client-parity` sınıfında
çalıştırır. Farklı gözlem, boş başlangıç, pozitif ve negatif packed koordinat
çözümü, living flag ile birlikte partial update, konumun korunması ve boş
Optional ile temizlenmesidir. Aynı sonucu veren mevcut `client-parity`
kayıtlarının sayısı **0**'dır; P08.z.1, P08.aa.1, P08.ad.1, P08.ae.1, P08.ag.1,
P08.ai.1, P08.ah.1, P08.aj.1, P08.ak.1, P08.al.1 ve P08.am.1 farklı metadata
alanlarını hedefler.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Living fixture'ı
  `living_flags=0` ve `is_sleeping=false` ile başlar; boş Optional Java JSON
  çıktısında `sleeping_pos` alanı bulunmaz.
- `0e0b01000003bffffe905aff`, ID `14`, serializer ID `11`, presence `1` ve
  packed `(14,90,-23)` konumunu taşır.
- `0800010e0b01ffff0000007fffc0ff`, living flags `0x01` ile packed
  `(-1024,-64,2047)` konumunu aynı metadata dizisinde taşır; iki state de
  uygulanır.
- `050801ff` yalnız no-gravity alanını taşır; sleeping position ve living
  flags önceki değerlerini korur.
- `0e0b00ff` geçerli boş Optional değeridir; sleeping position temizlenir,
  `is_sleeping=false` olur ve Java JSON'unda `sleeping_pos` alanı yine atlanır.
- Yatağa konumlandırma, sleeping pose/dimensions, bed facing/occupancy,
  movement reset'i, wake lifecycle ve local player parity'si bu state kaydının
  dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-sleeping-position`
kaydı Java oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`b44f92173726772293ce817465e8f919a5bf9c981238b7224f59803a6145e80d`

Math corpus'u **2988** kayıttır; fark `null` kalmıştır. Referansın gözlenen
durumları şöyledir:

| Adım | `living_flags` | `sleeping_pos` | `is_sleeping` |
| --- | ---: | --- | --- |
| initial | 0 | alan yok | false |
| set-positive-sleeping-position | 0 | `(14,90,-23)` | true |
| combined-living-and-negative-sleeping-position | 1 | `(-1024,-64,2047)` | true |
| no-gravity-preserves-sleeping-position | 1 | `(-1024,-64,2047)` | true |
| clear-sleeping-position | 1 | alan yok | false |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak ID `14`
`OPTIONAL_BLOCK_POS` metadata'sını çözer. `sleeping_pos_from_metadata`,
presence byte'ını ve packed BlockPos'ı tüketir; `ClientEntity::sleeping_pos` ve
`is_sleeping` state'leri world kopyasından sorgulanabilir. Adapter Java ile
aynı partial metadata sırasını uygular ve boş state için Java'nın atlanan JSON
alanını üretir.

Debug adapter SHA-256 değeri
`86560c2d0ae6d953807a20c9f0e92bd9a6d3f551c8ae95b9b9df78ddbac57c20` ile aynı
capture üzerinde **2988/2988** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/debug/examples/behavior_adapter
case_count=2988
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`468eec3a4ec3f96c6635127ecfa4cd3fc98ae4ddaaf8f5414012c52933f117b6` ile aynı
capture üzerinde **2988/2988** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/release/examples/behavior_adapter
case_count=2988
difference=null
status=parity_passed
```

Bu dilim serverbound paket üretmediği için vanilla sunucusuna ek bir bağlanma
sonucu Java client-parity karşılaştırmasına yeni kanıt eklemez; canlı sonuç
[P08.an server-conformance kaydında](client-world-entity-sleeping-position-conformance.md)
tutulur.

## Açık kapsam

Client-side `setPosToBed` konum snap'i, sleeping pose/dimensions/collision,
bed facing ve occupancy, wake/respawn lifecycle, villager ve diğer living
entity uyku dalları, local player parity'si, diğer living serializer'ları ve
oynanabilir istemci bağlantısı açık kalır.
