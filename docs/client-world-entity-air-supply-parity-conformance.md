# P08.aa.1 — ClientWorld entity air supply metadata — [client-parity]

Bu kayıt, Java istemcisinin entity metadata içindeki air-supply alanını
başlatmasını, signed VarInt olarak uygulamasını ve ayrı metadata güncellemesinde
önceki shared flags state'ini korumasını kapsar. Kanıt sınıfı
`client-parity`'dir: aynı özgün metadata trace'i Java 26.2 entity fixture'ına
ve production Rust `ClientWorld` adapter'ına verilir.

Önceki [P08.aa server-conformance kaydı](client-world-entity-air-supply-conformance.md)
vanilla sunucusunun bu metadata akışını kabul ettiğini gösterir. Bu kayıt onun
yerine geçmez; burada karşılaştırılan çıktı Java entity state'idir.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `client/multiplayer/ClientPacketListener.java:630-635`, entity metadata
  packet'ının hedef entity'ye `getEntityData().assignValues(...)` uyguladığını
  gösterir.
- `world/entity/Entity.java:263-271,304-321,2800-2809`, ID `1` air-supply
  alanının `INT` serializer'ı ile kurulduğunu, entity oluşturulurken
  `getMaxAirSupply()` sonucu olan `300` ile başladığını ve state'in doğrudan
  entity data'dan okunduğunu gösterir.
- `network/syncher/EntityDataSerializers.java:53-54,148-166`, `INT`
  serializer ID'sini `1` olarak verir; `SynchedEntityData.java:108-125,216-244`
  signed VarInt kaydının yalnız gönderilen data item'larını değiştirdiğini
  ve metadata sonlandırıcısını tanımlar.
- `world/entity/LivingEntity.java:436-456,587-600`, air supply azaltma ve
  artırma tick dallarını gösterir. Bu kayıt o tick gövdelerini değil, temel
  entity metadata state'ini ölçer.

## Yeni dal ve varyant kararı

Bu dilim, [P08.z.1 shared-flags client-parity kaydından](client-world-entity-shared-flags-parity-conformance.md)
ayrı olarak Java ID `1` `INT` assignment dalını ilk kez hedef state olarak
çalıştırır. Farklı gözlem, yeni entity'nin `300` varsayılanı, pozitif `250`
değerinin uygulanması, combined metadata içinde signed `-1` değerinin ve
shared flags'in birlikte taşınması, ardından yalnız flags güncellemesinde
`-1` değerinin korunmasıdır. Aynı sonucu veren mevcut `client-parity`
kayıtlarının sayısı **0**'dır; shared-flags kaydındaki `7` değeri yalnızca
partial assignment korumasını ölçer.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Yeni entity
  state'i `shared_flags=0`, `air_supply=300` ile başlar.
- `0101fa01ff`, ID `1`, `INT` serializer ID `1`, signed VarInt `250` ve
  `255` sonlandırıcısını taşır; sonuç `air_supply=250` olur.
- `0000400101ffffffff0fff`, ID `0` için `0x40` shared flags ve ID `1` için
  signed VarInt `-1` taşır; iki alan da uygulanır.
- `000000ff` yalnız ID `0`'ı günceller. Air supply `-1` olarak kalır ve shared
  flags sıfırlanır. Rust metadata state'i Java'nın bu partial assignment
  sırasını izler.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-air-supply` kaydı Java
oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`d0a1b665e6f2437c304d2abd70cf18f89cd8003c3929e811ef244106eb5b559e`

Math corpus'u **2971** kayıttır; fark `null` kalmıştır. Referansın gözlenen
durumları şöyledir:

| Adım | `shared_flags` | `air_supply` |
| --- | ---: | ---: |
| initial | 0 | 300 |
| set-positive-air-supply | 0 | 250 |
| combined-signed-air-supply | 64 | -1 |
| flags-only-preserves-air-supply | 0 | -1 |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak ID `1` signed
VarInt alanını çözer; `ClientEntity::air_supply` ve `shared_flags` state'leri
world kopyasından sorgulanabilir. Adapter başlangıç state'ini, combined
metadata uygulamasını ve partial update sonucunu Java ile karşılaştırır.

Debug adapter SHA-256 değeri
`9f1e622f05dcdd278dd72625ef3cdf02d19f3679806e4ecdbfc9e05206dc3511` ile aynı
capture üzerinde **2971/2971** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/debug/examples/behavior_adapter
case_count=2971
difference=null
status=parity_passed

cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/release/examples/behavior_adapter
case_count=2971
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`b14ba91baa9e5570f5515b518a21891273ea745527315bd6290715ae0d4e2ea6` ile aynı
capture üzerinde **2971/2971** eşleşti; fark `null` kaldı.

## Açık kapsam

Bu kayıt entity türlerinin `getMaxAirSupply()` override'larını, breathing ve
drowning tick yan etkilerini, su/bubble-column algısını, oxygen bonusunu,
air HUD/render state'ini, local player özel yaşam döngüsünü, diğer metadata
serializer'larını ve oynanabilir istemci bağlantısını kapsamaz. Vanilla
sunucusuna canlı bağlantı bu `client-parity` kanıtının yerine geçmez.
