# P08.ae.1 — ClientWorld entity freeze progress — [client-parity]

Bu kayıt, Java entity metadata içindeki `ticksFrozen` alanının Rust
`ClientWorld` kopyasına uygulanmasını ve Java'nın freeze türevlerini aynı
sentetik metadata dizisiyle karşılaştırır. Kanıt sınıfı `client-parity`'dir:
Java 26.2 entity fixture'ı ile production Rust adapter aynı girdiyi alır.

Önceki [P08.ae server-conformance kaydı](client-world-entity-ticks-frozen-conformance.md)
Mojang vanilla 26.2 dedicated server'ın bu metadata akışını kabul ettiğini
gösterir. Bu kayıt, canlı sunucu wire uyumluluğundan ayrı olarak istemci
state'inin Java ile eşitliğini ölçer.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `world/entity/Entity.java:271,320`, ID `7` alanının `INT` serializer ile
  `0` varsayılanını kurduğunu gösterir.
- `world/entity/Entity.java:2812-2833`, signed metadata değerini okur;
  `getPercentFrozen()` için `min(ticksFrozen, getTicksRequiredToFreeze())`
  hesabını, `isFullyFrozen()` için aynı eşiği kullanır. Temel entity'nin
  `getTicksRequiredToFreeze()` sonucu `140`'tır.
- `world/entity/Entity.java:3881-3884`, `isFreezing()` sonucunu pozitif
  `ticksFrozen` değerine bağlar.
- `network/syncher/EntityDataSerializers.java:53-54,148-166` ve
  `network/protocol/game/ClientboundSetEntityDataPacket.java:21-37`, ID `7`
  değerinin signed VarInt olarak metadata sonlandırıcısından önce taşındığını
  ve yalnız gönderilen data item'larının atandığını doğrular.

## Yeni dal ve varyant kararı

Bu dilim, önceki P08.ae server-conformance kaydındaki Java ID `7` metadata
dalını ilk kez `client-parity` sınıfında çalıştırır. Yeni gözlem, aynı entity
state'i üzerinde `70`, eşik değeri `140`, signed `-1`, combined shared flag ve
ardından yalnız freeze alanını temizleyen partial metadata adımlarının Java ve
Rust'ta aynı türevleri üretmesidir. Aynı sonucu veren mevcut `client-parity`
kayıtlarının sayısı **0**'dır; P08.z.1, P08.aa.1 ve P08.ad.1 farklı metadata
alanlarını hedefler.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir.
- Yeni entity `ticks_frozen=0` ile başlar; `percent_frozen` Java f32 bitleri
  ile raporlanır.
- `070146ff`, ID `7`, INT serializer ID `1` ve `70` değerini taşır. Sonuç
  `ticks_frozen=70`, `is_freezing=true`, `is_fully_frozen=false` ve yüzde
  bitleri `3f000000`'dır.
- `07018c01ff` eşik değeri `140`'ı taşır. Sonuç yüzde bitleri `3f800000` ve
  `is_fully_frozen=true` olur.
- `0000400701ffffffff0fff`, shared flags `64` ile freeze değerini `-1` olarak
  birlikte yazar. Sonuç `is_freezing=false`, `is_fully_frozen=false` ve yüzde
  bitleri `bbea0ea1` olur.
- `070100ff` yalnız freeze alanını `0` yapar; önceki shared flags `64` korunur.

## Java yakalama

`tools/behavior/cases.py` içindeki `client-world/entity-ticks-frozen` kaydı
Java oracle'ında iki çalıştırmada kararlı kaldı.

Java referans kimliği:
`7292b60ea457c25ab02fbd8f713430b6fcc758b087578f0c208148b652609387`

Math corpus'u **2980** kayıttır; fark `null` kalmıştır. Referans çıktısındaki
freeze state'leri şöyledir:

| Adım | `shared_flags` | `ticks_frozen` | `percent_frozen` f32 bitleri | `is_freezing` | `is_fully_frozen` |
| --- | ---: | ---: | --- | --- | --- |
| initial | 0 | 0 | `00000000` | false | false |
| set-positive-freeze | 0 | 70 | `3f000000` | true | false |
| set-full-freeze | 0 | 140 | `3f800000` | true | true |
| combined-shared-flags-and-negative-freeze | 64 | -1 | `bbea0ea1` | false | false |
| clear-freeze | 64 | 0 | `00000000` | false | false |

## Rust doğrulaması

`ClientWorld::apply_entity_metadata` production API olarak ID `7` INT
metadata'sını çözer. `ClientEntity` state'i `ticks_frozen`, yüzde ve iki
boolean türevi dışarı verir; adapter Java ile aynı metadata sırasını uygular.
Yüzde karşılaştırması ondalık yazım farklarını gizlememek için raw f32 bitleri
üzerinden yapılır.

Debug adapter SHA-256 değeri
`968efd784290661431cf0fc72970d99ee8623c244f1351d3747e03045dd4b144` ile aynı
capture üzerinde **2980/2980** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/debug/examples/behavior_adapter
case_count=2980
difference=null
status=parity_passed
```

Release adapter SHA-256 değeri
`2ae2975500e25e468aa28e68a791dbdf9201c6abcea0cdcf6b0cf4dc3ca3bb37` ile aynı
capture üzerinde **2980/2980** eşleşti; fark `null` kaldı.

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/release/examples/behavior_adapter
case_count=2980
difference=null
status=parity_passed
```

Vanilla 26.2 server üzerindeki önceki P08.ae probe'u
[server-conformance kaydında](client-world-entity-ticks-frozen-conformance.md)
tutulur. Bu dilim serverbound paket üretmediği için canlı vanilla sunucusuna
ek bir bağlanma sonucu bu Java client-parity karşılaştırmasına yeni kanıt
eklemez.

## Açık kapsam

Bu kayıt LivingEntity freeze azaltma ve hasar tick'ini, powder snow HUD ve
renderer etkilerini, türün `getTicksRequiredToFreeze()` override'larını, tam
metadata serializer setini ve client tick/lifecycle entegrasyonunu ölçmez.
Bu alanlar için aynı Java client-parity kanıtı ayrıca gereklidir.
