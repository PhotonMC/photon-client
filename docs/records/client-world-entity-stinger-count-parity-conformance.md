# P08.ao.1 — ClientWorld entity living stinger count — [client-parity]

Bu kayıt, Java `LivingEntity` metadata içindeki ID `13` stinger count INT
alanının Rust `ClientWorld` kopyasına uygulanmasını ve signed VarInt state'inin
aynı sentetik metadata dizisinde Java ile eşleşmesini kapsar. Kanıt sınıfı
`client-parity`'dir: Java 26.2 living entity fixture'ı ile production Rust
adapter aynı girdiyi alır.

Önceki [P08.ao server-conformance kaydı](client-world-entity-stinger-count-conformance.md)
Mojang vanilla 26.2 dedicated server'ın arı iğnesi metadata akışını kabul
ettiğini gösterir. Bu kayıt canlı sunucu wire uyumluluğundan ayrı olarak
istemci world-copy state'inin Java ile eşitliğini ölçer.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `world/entity/LivingEntity.java:176-186,322-329`, stinger count alanının
  arrow count'tan ayrı ID `13`, `INT` serializer ve `0` başlangıçla
  tanımlandığını gösterir.
- `world/entity/LivingEntity.java:2008-2013`, getter ve setter'ın synchronized
  entity data değerini doğrudan kullandığını tanımlar.
- `world/entity/LivingEntity.java:2769-2779`, server tick'inde pozitif stinger
  sayısının zamanla azaltılabildiğini gösterir. Bu kayıt metadata assignment
  state'ini ölçer.
- `world/entity/animal/bee/Bee.java:230`, başarılı bee sting sonrasında yaşayan
  hedefin sayısının bir artırıldığı dalı; `client/renderer/entity/player/AvatarRenderer.java:17,173`
  ve `client/renderer/entity/layers/BeeStingerLayer.java:12-27`, sayının render
  state'ine taşındığı yolu doğrular.
- `network/syncher/EntityDataSerializers.java:53-54,164-172` ve
  `network/codec/ByteBufCodecs.java:106-113`, INT serializer ID'sinin `1` ve
  signed VarInt wire biçiminin kullanıldığını doğrular.
- `network/syncher/SynchedEntityData.java:74-85,108-125` ve
  `network/protocol/game/ClientboundSetEntityDataPacket.java:21-37`, yalnız
  gönderilen alanların atanmasını ve `255` metadata sonlandırıcısını tanımlar.

Java oracle'ı gerçek `LivingEntity` getter'larını kullanan `ArmorStand` tabanlı
özgün fixture çalıştırır. Arrow count ile aynı primitive codec kullanılsa da
ID `13` ve BeeStingerLayer tüketicisi ayrı Java state dalıdır.

## Yeni dal ve varyant kararı

Bu dilim, önceki P08.ao server-conformance kaydındaki Java ID `13` INT
assignment dalını ilk kez `client-parity` sınıfında çalıştırır. Farklı gözlem,
stinger count'un `0` başlangıcı, pozitif değer, signed negatif VarInt, başka
metadata ile partial korunması ve temizlenmesidir. Aynı sonucu veren mevcut
`client-parity` kayıtlarının sayısı **0**'dır; P08.ak.1 arrow count için ID `12`
alanını, P08.ai.1 living flags'i ve diğer P08 parity kayıtları farklı metadata
alanlarını hedefler.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Living fixture'ı
  `stinger_count=0` ve `living_flags=0` ile başlar.
- `0d0102ff`, ID `13`, INT serializer ID `1` ve `2` değerini taşır.
- `0800010d01ffffffff0fff`, living flags `0x01` ile signed VarInt `-1`
  stinger count değerini aynı metadata dizisinde taşır.
- `050801ff` yalnız no-gravity metadata'sını taşır; stinger count `-1` ve
  living flags `1` olarak kalır.
- `0d0100ff` stinger count'u `0` yapar; living flags `1` olarak korunur.
- Bee sting/poison, count'un server tick'iyle azalması, BeeStingerLayer render'ı,
  equipment/model state'i ve local player parity'si bu metadata state kaydının
  dışındadır.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-stinger-count` kaydı
Java oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`bf33139dfbe001fb02f9a3936ef45a355f0923d86863b4934d836779ee8f8239`

Math corpus'u **2989** kayıttır; fark `null` kalmıştır. Referansın gözlenen
durumları şöyledir:

| Adım | `stinger_count` | `living_flags` |
| --- | ---: | ---: |
| initial | 0 | 0 |
| set-positive-stinger-count | 2 | 0 |
| combined-living-and-negative-stinger-count | -1 | 1 |
| no-gravity-preserves-stinger-count | -1 | 1 |
| clear-stinger-count | 0 | 1 |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak ID `13` INT
metadata'sını çözer. `ClientEntity::stinger_count` signed `i32` state'ini
sorgular; adapter Java ile aynı metadata sırasını uygular.

Debug adapter SHA-256 değeri
`e4147c3da026414f46526dfd372674628382b20e5330613b8287298e956680bc` ile aynı
capture üzerinde **2989/2989** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/debug/examples/behavior_adapter
case_count=2989
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`9fd51f83390af4a61e1531566536789e4f45c3b2e5e48380b7f44c88889a7ce0` ile aynı
capture üzerinde **2989/2989** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/release/examples/behavior_adapter
case_count=2989
difference=null
status=parity_passed
```

Vanilla 26.2 server üzerindeki önceki P08.ao probe'u
[server-conformance kaydında](client-world-entity-stinger-count-conformance.md)
tutulur. Bu dilim serverbound paket üretmediği için canlı vanilla sunucusuna
ek bir bağlanma sonucu Java client-parity karşılaştırmasına yeni kanıt eklemez.

## Açık kapsam

Stinger count'un server tick'iyle azalması, birden fazla sting, bee poison ve AI,
remote/local living entity render katmanları, stinger model/texture, local
player parity'si, diğer living serializer'ları ve oynanabilir istemci bağlantısı
açık kalır.
