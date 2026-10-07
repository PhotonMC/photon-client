# P08.z.1 — ClientWorld entity shared flags metadata — [client-parity]

Bu kayıt, Java istemcisinin entity metadata içindeki ortak flags byte'ını
uygulamasını ve yalnız gönderilen metadata alanını değiştirmesini kapsar.
Kanıt sınıfı `client-parity`'dir: aynı özgün metadata trace'i Java 26.2
entity fixture'ına ve production Rust `ClientWorld` adapter'ına verilir.

Önceki [P08.z server-conformance kaydı](client-world-entity-shared-flags-conformance.md)
vanilla sunucusunun metadata akışını kabul ettiğini gösterir. Bu kayıt onun
yerine geçmez; burada karşılaştırılan çıktı Java entity state'idir.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `client/multiplayer/ClientPacketListener.java:630-635`, entity metadata
  packet'ının hedef entity'yi bulup `getEntityData().assignValues(...)`
  çağırdığını gösterir.
- `world/entity/Entity.java:255-271,304-321,2661-2750,2787-2809`, bütün
  entity'lerin ID `0` ortak `BYTE` flags alanını sıfırla başlattığını ve
  on-fire, shift, sprinting, swimming, invisible, glowing durumlarını bu
  byte'ın bitlerinden okuduğunu gösterir. Living entity fall-flying için aynı
  shared bitini kullanır.
- `network/syncher/SynchedEntityData.java:108-125,216-244` yalnız gelen
  `DataValue` kayıtlarının atanmasını ve metadata byte/serializer/değer/
  `255` sonlandırıcı biçimini tanımlar.
- `network/protocol/game/ClientboundSetEntityDataPacket.java:21-37`, packet'ın
  packed metadata listesini entity state'ine taşıyan katmandır.

## Yeni dal ve varyant kararı

Bu dilim, önceki yalnız `server-conformance` kaydından ayrı olarak Java
`handleSetEntityData` → `SynchedEntityData.assignValues` dalını ilk kez
`client-parity` ile çalıştırır. Farklı gözlem, ID `0` ile yedi ortak bitin
entity state'ine uygulanması, yalnız ID `1` air-supply kaydının ortak flags'i
koruması ve sonraki ID `0` kaydının tüm bitleri temizlemesidir. Aynı sonucu
veren mevcut `client-parity` kayıtlarının sayısı **0**'dır.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Entity fixture'ı
  yeni bir entity olarak sıfır shared flags ve `300` air supply ile başlar.
- Metadata `0000fbff`, ID `0`, `BYTE` serializer ID `0`, `0xfb` değerini ve
  `255` sonlandırıcısını taşır. `0xfb`, Java'nın tanımlı yedi ortak bitinin
  tamamını açar; shared flags sorgusu `251` ve tüm named bit sorguları `true`
  olur.
- Metadata `010107ff`, yalnız ID `1`, `INT` serializer ID `1` ve `7` air
  supply değerini taşır. Java'nın partial assignment davranışı nedeniyle
  shared flags `251` olarak kalır ve air supply `7` olur.
- Metadata `000000ff` ortak flags'i sıfırlar; air supply `7` olarak kalır.
  State sorgusu raw byte'ı ve yedi named türevi karşılaştırır.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-shared-flags` kaydı
Java oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`ef746bab072e922a670737f7cf728de0f18949395aca47eed1a262f71d94ea08`

Math corpus'u **2970** kayıttır; fark `null` kalmıştır. Referansın gözlenen
durumları şöyledir:

| Adım | `shared_flags` | named flags | `air_supply` |
| --- | ---: | --- | ---: |
| all-defined-flags | 251 | yedisi `true` | 300 |
| air-supply-only-preserves-flags | 251 | yedisi `true` | 7 |
| clear-shared-flags | 0 | yedisi `false` | 7 |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak entity metadata
gövdesini çözer. `ClientEntity` shared flags ve air supply state'ini, ayrıca
Java bit maskelerinin yedi named sorgusunu sağlar. Adapter aynı üç metadata
gövdesini uygular ve Java state alanlarını karşılaştırır.

Debug adapter SHA-256 değeri
`8bcde667d33499f52ff81c6297388263dde966f6c02e8bdfd4073a92d406150f` ile aynı
capture üzerinde **2970/2970** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/debug/examples/behavior_adapter
case_count=2970
difference=null
status=parity_passed

cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/release/examples/behavior_adapter
case_count=2970
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`d98e9f5b2e015463c4f166021ccda4f3d83f59759a34f1d75431645b4970a94c` ile aynı
capture üzerinde **2970/2970** eşleşti; fark `null` kaldı.

## Açık kapsam

Bu kayıt diğer metadata serializer'larını, entity türüne göre metadata ID
doğrulamasını, `onSyncedDataUpdated` yan etkilerini, living effect state'inin
tam istemci kopyasını, pose/name/equipment render davranışını ve oynanabilir
istemci bağlantısını kapsamaz. Vanilla sunucusuna canlı bağlantı bu
`client-parity` kanıtının yerine geçmez.
