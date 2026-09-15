# P10.f.1 — LocalPlayer movement speed attribute in water travel — [client-parity]

Bu kayıt, Java `LivingEntity.travelInWater` içindeki water movement
efficiency dalının `Player.getSpeed()` sonucunu tüketmesini local player
ClientWorld motion state'ine bağlar. Kanıt sınıfı `client-parity`'dir: aynı
özgün sentetik trace, Java Edition 26.2 oracle'ına ve production Rust
`ClientWorld` yoluna verildi. Yeni bir wire veya canlı sunucu davranışı
eklenmedi.

## Java kaynak incelemesi

Pinned Java Edition 26.2, protokol 776 ve Java 25 kaynaklarında şu yollar
esas alındı:

- `world/entity/LivingEntity.java:2519-2526`, pozitif water movement
  efficiency için başlangıç speed'i `0.02F` değerinden `getSpeed()` yönünde
  f32 işlemiyle karıştırır.
- `world/entity/LivingEntity.java:2532-2541`, relative hareketi, su drag'ını
  ve fluid falling adjustment sırasını yürütür.
- `world/entity/player/Player.java:1445-1448`, çağrılan `getSpeed()` override'ı
  sonucunu `Attributes.MOVEMENT_SPEED` effective değerinden alır.
- `world/entity/ai/attributes/AttributeInstance.java:139-170`, bu effective
  değerin modifier operation sırasını ve range sanitize yolunu belirler.

Oracle, `Player.createAttributes().build()` ile gerçek `AttributeMap` kurar
ve bunu kontrollü bir `LocalPlayer` instance'ına bağlar. Gerçek protected
`travelInWater` metodu reflection ile çağrılır; fixture yalnızca su ortamı,
motion ve collision sonuçlarını sağlar. Böylece water efficiency dalındaki
`getSpeed()` çağrısı Player attribute state'inden okunur.

## Yeni dal ve varyant kararı

- İlk kez çalışan Java dalı, `travelInWater` içindeki
  `waterWalker > 0.0F` koşulunun `speed += (getSpeed() - speed) *
  waterWalker` hesabıdır. Bu çağrı artık çözümlenmiş speed parametresi yerine
  ClientWorld local player attribute state'ine bağlıdır.
- Öncekilerden farklı gözlem, aynı su trace'inde Player speed'in varsayılan
  `0.1F`, modifier'lı `0.63` ve replacement sonrası `0.4` olmasıyla motion
  vektörünün değişmesi; yanlış player ID'sinde replacement hızının korunmasıdır.
- Aynı sonucu veren mevcut kayıtların sayısı **0**'dır. Water travel parity
  kaydı speed'i dışarıdan çözümlenmiş alır; grounded attribute speed kaydı ise
  water travel çağrısını çalıştırmaz.

## Davranış sözleşmesi

- Local player ID'si `42` ile eşleşen snapshot `movement_speed` state'ini
  günceller; başka ID'ye ait snapshot local player hızını değiştirmez.
- `water_efficiency=0.6F` ve grounded başlangıçta travel speed'i Java'nın
  `0.02F` değerinden Player `getSpeed()` sonucuna doğru karıştırılır.
- Relative input mevcut delta'ya eklenir; ardından su X/Z drag'ı, Y `0.8F`
  drag'ı ve fluid falling adjustment Java sırasıyla uygulanır.
- Bu dilim yalnız `movement_speed` attribute'unun water efficiency yoluna
  bağlanmasını kapsar. Gerçek fluid lookup, water efficiency attribute'ının
  registry binding'i, collision-aware `move` ve LocalPlayer tick akışı açık
  kalır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/player-attribute-water-travel-binding` trace'i aynı input,
delta ve water efficiency ile dört ardışık state taşır:

| Gözlem | Kabul | Player speed | Motion X | Motion Y | Motion Z |
| --- | --- | --- | --- | --- | --- |
| `default-speed-enters-water-efficiency` | `true` | `3dcccccd` | `3fab8b47e36ce9e4` | `bfc7e7c07674d164` | `3fcd7991a0f9a2b9` |
| `modified-speed-enters-water-efficiency` | `true` | `3f2147ae` | `3f64b75e6f6bcd91` | `bfd2771ca8605686` | `3fd98280957d57d7` |
| `replacement-speed-enters-water-efficiency` | `true` | `3ecccccd` | `3f995f3814d5e777` | `bfcf473045197a24` | `3fd4d5c1ae6732af` |
| `wrong-player-preserves-water-speed` | `false` | `3ecccccd` | `3f995f3814d5e777` | `bfcf473045197a24` | `3fd4d5c1ae6732af` |

Java referans kimliği:
`d811840eb419477417562931027c674333eb2edeb4d6317d495dae75571d37f1`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2974** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`ClientWorld::apply_local_player_travel_in_water_from_attributes`, mevcut
local player motion state'ini koruyarak `ClientPlayerState::movement_speed`
sonucunu `LocalPlayerTravelInWaterParameters.speed` alanına yerleştirir ve
aynı production water helper'ını çağırır. Parametreli mevcut API bu değişiklik
olmadan kalır.

Debug adapter SHA-256 değeri
`db3639bff7a515cc71f9235e54757d6c6c3caec8a415d3cd176d8d75df95a3e2` ile aynı
capture üzerinde **2974/2974** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`2203bdff20e01958f71e15b1d9228ab740679afd721800e397fa0a32e5d2d2b5` ile aynı
capture üzerinde **2974/2974** eşleşti; fark `null` kaldı.

Çalışan kontroller:

```text
cargo test --locked --offline --quiet
190 passed; 0 failed

cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --all -- --check

python3 tools/behavior/run.py compare --family math \
  --jdk-home /usr/lib/jvm/java-25-openjdk \
  --adapter target/debug/examples/behavior_adapter
case_count=2974
difference=null
status=parity_passed

python3 tools/behavior/run.py compare --family math \
  --jdk-home /usr/lib/jvm/java-25-openjdk \
  --adapter target/release/examples/behavior_adapter
case_count=2974
difference=null
status=parity_passed
```

## Açık kapsam

`movement_speed` attribute'unun LocalPlayer gerçek tick akışına, hava ve
diğer travel dallarına, sprint transition/effect/item modifier yaşam
döngüsüne bağlanması; water movement efficiency'nin tam attribute registry
lookup'ı; collision-aware hareket, serverbound movement paketleri ve
oynanabilir istemci açıktır.
