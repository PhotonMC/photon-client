# P08.ad.1 — ClientWorld entity pose metadata — [client-parity]

Bu kayıt, Java istemcisinin entity metadata içindeki pose alanını çözmesini,
combined metadata ile diğer state'i korumasını ve bilinmeyen pose ID'sini
`STANDING`e düşürmesini kapsar. Kanıt sınıfı `client-parity`'dir: aynı özgün
metadata trace'i Java 26.2 entity fixture'ına ve production Rust
`ClientWorld` adapter'ına verilir.

Önceki [P08.ad server-conformance kaydı](client-world-entity-pose-conformance.md)
vanilla sunucusunun pose metadata akışını kabul ettiğini gösterir. Bu kayıt
onun yerine geçmez; burada karşılaştırılan çıktı Java entity state'idir.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `client/multiplayer/ClientPacketListener.java:630-635`, entity metadata
  packet'ının hedef entity'ye `getEntityData().assignValues(...)` uyguladığını
  gösterir.
- `world/entity/Entity.java:270-271,319,440-449,3377-3380`, ID `6` pose
  alanının `POSE` serializer'ı ile `STANDING` başladığını ve gelen pose
  değerinde `refreshDimensions()` çağrısının tetiklendiğini gösterir.
- `world/entity/Pose.java:9-37`, pose ID `0`–`17` tablosunu ve aralık dışı
  değerlerin `STANDING` fallback'ini tanımlar.
- `network/syncher/EntityDataSerializers.java:117,184` ile
  `network/codec/ByteBufCodecs.java:546-558`, `POSE` serializer ID `20` ve
  pose ID'sinin VarInt wire biçimini verir. `SynchedEntityData.java:108-125`
  yalnız gelen data item'larının uygulanmasını sağlar.

## Yeni dal ve varyant kararı

Bu dilim, önceki [P08.ad server-conformance kaydından](client-world-entity-pose-conformance.md)
ayrı olarak Java ID `6` `POSE` assignment dalını ilk kez `client-parity` ile
çalıştırır. Farklı gözlem, geçerli pose ID'sinin state'e yazılması, combined
metadata içindeki ortak flags alanının birlikte uygulanması ve ID `99`'un
`STANDING`e dönmesidir. Aynı sonucu veren mevcut `client-parity` kayıtlarının
sayısı **0**'dır; diğer parity kayıtları pose alanını hedeflemez.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Yeni entity pose
  değeri `STANDING`, wire ID `0` ile başlar.
- `061405ff`, ID `6`, `POSE` serializer ID `20`, pose ID `5` ve `255`
  sonlandırıcısını taşır; sonuç `CROUCHING (5)` olur.
- `00004006140aff`, ID `0` için `0x40` shared flags ve ID `6` için pose ID
  `10` taşır; sonuç shared flags `64` ve `SITTING (10)` olur.
- `061463ff`, pose ID `99` taşır. Java `Pose.BY_ID` fallback'i sonucu pose
  `STANDING (0)` olur; önceki shared flags `64` olarak korunur.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-pose` kaydı Java
oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`528f0f4de3a49010491f6e8c27594911b4b9fb443015734b07bf2dae083764f0`

Math corpus'u **2972** kayıttır; fark `null` kalmıştır. Referansın gözlenen
durumları şöyledir:

| Adım | `shared_flags` | `pose_id` |
| --- | ---: | ---: |
| initial | 0 | 0 |
| set-crouching | 0 | 5 |
| set-sitting-with-glowing | 64 | 10 |
| unknown-pose-falls-back | 64 | 0 |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak ID `6` pose
metadata'sını çözer. `ClientEntityPose::from_id` Java pose ID tablosunu ve
`STANDING` fallback'ini taşır; `ClientEntity::pose` ve shared flags state'leri
world kopyasından sorgulanabilir. Adapter aynı metadata sırasını Java ile
karşılaştırır.

Debug adapter SHA-256 değeri
`e50ace7badd9893daa8bec944902d69f25316ab9e05c9bca8b98374670f57056` ile aynı
capture üzerinde **2972/2972** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/debug/examples/behavior_adapter
case_count=2972
difference=null
status=parity_passed

cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/release/examples/behavior_adapter
case_count=2972
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`33e62cb0bc7cfabad748f00c5cf170b6551b1b4de2b8c9bfcece759643c9bf35` ile aynı
capture üzerinde **2972/2972** eşleşti; fark `null` kaldı.

## Açık kapsam

Bu kayıt `refreshDimensions()` sonucunu, pose'a bağlı bounding box/collision,
movement, camera ve render etkilerini, entity türüne özel pose geçişlerini,
local player pose yaşam döngüsünü ve diğer metadata serializer'larını ölçmez.
Vanilla sunucusuna canlı bağlantı bu `client-parity` kanıtının yerine geçmez.
