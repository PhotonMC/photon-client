# P10.f.2 — LocalPlayer sprint transition and movement speed attribute — [client-parity]

Bu kayıt, Java `LivingEntity.setSprinting` geçişini local player
`ClientWorld` state'ine bağlar. Kanıt sınıfı `client-parity`'dir: aynı özgün
sentetik trace, Java Edition 26.2 oracle'ına ve production Rust
`ClientWorld` yoluna verildi. Canlı sunucu veya Pumpkin sonucu kullanılmadı.

## Java kaynak incelemesi

Pinned Java Edition 26.2, protokol 776 ve Java 25 kaynaklarında şu yollar
esas alındı:

- `world/entity/Entity.java:2710-2715`, sprinting sorgusunun shared flag bit
  `3`'ü okuduğunu ve setter'ın bu biti güncellediğini gösterir.
- `world/entity/LivingEntity.java:153-155`, sprint modifier'ını
  `minecraft:sprinting`, `0.3F` ve `ADD_MULTIPLIED_TOTAL` olarak tanımlar.
- `world/entity/LivingEntity.java:2315-2323`, `setSprinting` içinde önce
  `super.setSprinting`, sonra mevcut sprint modifier'ının kaldırılması ve
  sprint açıksa transient modifier'ın yeniden eklenmesi sırasını tanımlar.

Oracle, `Player.createAttributes().build()` ile gerçek `AttributeMap` kurar,
bu map'i kontrollü bir `LocalPlayer` fixture'ına bağlar ve gerçek
`LivingEntity.setSprinting` çağrısını çalıştırır. Fixture'ın shared flag
verisi, Entity constructor'ının gerçek `SynchedEntityData` state'inden alınır.

## Yeni dal ve varyant kararı

- İlk kez çalışan Java dalı, `LivingEntity.setSprinting(boolean)` içindeki
  movement speed attribute modifier yaşam döngüsüdür: setter çağrısından önce
  aynı identifier'ın kaldırılması ve yalnızca `true` dalında yeniden eklenmesi.
- Öncekilerden farklı gözlem, sprint flag'i ile birlikte movement speed'in
  varsayılan `0.1F` taban üzerinde `minecraft:sprinting` modifier'ı ile
  değişmesi; tekrar `true` çağrısının duplicate üretmemesi; `false` çağrısının
  modifier'ı kaldırırken özel modifier'ı korumasıdır.
- Aynı sonucu veren mevcut kayıtların sayısı **0**'dır. Önceki attribute
  kayıtları yalnızca snapshot uygulamasını ve attribute değerinin grounded
  veya water travel tüketimini çalıştırır; sprint transition çağrısını ve
  shared flag güncellemesini çalıştırmaz.

## Davranış sözleşmesi

- Eşleşen local player ID'sinde `setSprinting(true)`, shared flag bit `3`'ü
  set eder ve `minecraft:sprinting` modifier'ını operation `2` ile tekil
  olarak ekler.
- `minecraft:sprinting` modifier'ı tekrar eklenmeden önce identifier'ına
  göre kaldırılır; art arda `true` çağrıları birden fazla modifier üretmez.
- `setSprinting(false)` shared flag bit `3`'ü temizler ve yalnız sprint
  modifier'ını kaldırır; diğer movement speed modifier'ları korunur.
- Java kaynağındaki `0.1F` ve `0.3F` literal'larının attribute map'e double
  olarak taşınan bitleri korunur; `Player.getSpeed()` sonucu f32 olarak
  sorgulanır.
- Farklı player ID'si ile gelen transition ve snapshot local player state'ini
  değiştirmez.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/player-sprint-transition-attribute-binding` trace'i beş
ardışık state taşır:

| Gözlem | Kabul | Sprint | Base value | Modifier state | Movement speed |
| --- | --- | --- | --- | --- | --- |
| `default-start-sprinting` | `true` | `true` | `3fb99999a0000000` | `minecraft:sprinting:3fd3333340000000:2` | `3e051eb9` |
| `repeated-start-sprinting` | `true` | `true` | `3fb99999a0000000` | `minecraft:sprinting:3fd3333340000000:2` | `3e051eb9` |
| `custom-speed-start-sprinting` | `true` | `true` | `3fd0000000000000` | `minecraft:sprinting:3fd3333340000000:2`, `p10:sprint_custom:3fc999999999999a:2` | `3ec7ae15` |
| `stop-sprinting-removes-modifier` | `true` | `false` | `3fd0000000000000` | `p10:sprint_custom:3fc999999999999a:2` | `3e99999a` |
| `wrong-player-preserves-stopped-state` | `false` | `false` | `3fd0000000000000` | `p10:sprint_custom:3fc999999999999a:2` | `3e99999a` |

Java referans kimliği:
`da494490e77c6dca7ef8edab4c8c35ad36dd65ef784f820735821d6af6070994`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2975** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`ClientWorld::apply_local_player_sprinting`, eşleşen local player için
`ClientPlayerState` shared flag'ini günceller, movement speed attribute'ındaki
aynı identifier'lı modifier'ları temizler ve sprint açıksa Java operation
`2` modifier'ını ekler. Attribute yoksa Java Player default movement speed
state'i oluşturulur; mevcut custom attribute ve modifier'lar korunur.

Debug adapter SHA-256 değeri
`cb326f896688268358457b1f5edb993ee6e83141767aecd17335ea1f02130898a` ile
aynı capture üzerinde **2975/2975** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`a85b873d4acf5ccf0fbd06bef5a2e0b0bd29b63e836b9500f6406b7003b0daaf` ile
aynı capture üzerinde **2975/2975** eşleşti; fark `null` kaldı.

Çalışan kontroller:

```text
cargo test local_player_sprinting_updates_shared_flag_and_movement_speed_modifier
1 passed; 0 failed

python3 tools/behavior/run.py compare --family math \
  --jdk-home /usr/lib/jvm/java-25-openjdk \
  --adapter target/debug/examples/behavior_adapter
case_count=2975
difference=null
status=parity_passed

python3 tools/behavior/run.py compare --family math \
  --jdk-home /usr/lib/jvm/java-25-openjdk \
  --adapter target/release/examples/behavior_adapter
case_count=2975
difference=null
status=parity_passed
```

## Açık kapsam

Keyboard sprint input'ının `LocalPlayer` tick akışında transition'a
bağlanması, sprint değişim paketleri, effect/item kaynaklı attribute
değişimleri, tam attribute synchronization ve oynanabilir istemci açıktır.
