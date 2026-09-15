# P10.f — LocalPlayer movement speed attribute binding — [client-parity]

Bu kayıt, Java `Player.getSpeed()` sonucunun local player hareketindeki
ground friction yoluna bağlanmasını kapsar. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik trace, Java Edition 26.2 oracle'ına
ve production Rust `ClientWorld` yoluna verildi. Bu dilimde wire veya canlı
sunucu davranışı değişmediği için ayrıca `server-conformance` koşusu
yapılmadı.

## Java kaynak incelemesi

Pinned Java Edition 26.2, protokol 776 ve Java 25 kaynaklarında şu dallar
esas alındı:

- `world/entity/player/Player.java:206-210`, Player attribute supplier'ının
  `movement_speed` başlangıç değerini `0.1F` olarak kurar.
- `world/entity/player/Player.java:1445-1448`, Player'ın `getSpeed()` override'ı
  sonucunu `Attributes.MOVEMENT_SPEED` attribute'undan okur.
- `world/entity/LivingEntity.java:2719-2724`, zeminde block friction eşiğine
  göre bu hızı doğrudan veya `0.21600002F / friction^3` katsayısıyla kullanır.
- `world/entity/ai/attributes/AttributeInstance.java:139-170`, effective
  attribute değerini Java operation sırasıyla hesaplar ve range sanitize
  uygular.

Oracle, `Player.createAttributes().build()` ile gerçek `AttributeMap` kurar.
Kontrollü bir `LocalPlayer` instance'ının private attribute map'i bu state'e
bağlanır; çağrı doğrudan Player'ın `getSpeed()` override'ından ve
`LivingEntity.getFrictionInfluencedSpeed(float)` reflection yolundan geçer.

## Yeni dal ve varyant kararı

- İlk kez çalışan Java dalı, `Player.getSpeed()` override'ının grounded
  `LivingEntity.getFrictionInfluencedSpeed` çağrısında
  `Attributes.MOVEMENT_SPEED` değerini okumasıdır.
- Öncekilerden farklı gözlem, attribute snapshot uygulanmadan Player default
  hızının `0.1F` olması; üç operation modifier'ı ile `0.63`'e çıkması;
  replacement snapshot'ı ile `0.4`'e dönmesi ve yanlış player ID'sinin önceki
  hızı korumasıdır. Aynı state'in block friction çıktısı da karşılaştırılır.
- Aynı sonucu veren mevcut kayıtların sayısı **0**'dır. Attribute parity kaydı
  snapshot/effective query ile, friction parity kaydı ise hızı çağıranından
  çözümlenmiş olarak alır; ikisi Player attribute map'ini hareket yoluna
  bağlayan bu dalı çalıştırmaz.

## Davranış sözleşmesi

- Local player ID'si `42` ile eşleşen attribute snapshot'ı
  `movement_speed` instance'ının base ve modifier state'ini değiştirir;
  eşleşmeyen ID local state'i değiştirmez.
- Attribute bulunmadığında bu kısmi ClientWorld state'i Java Player'ın
  `0.1F` başlangıç hızını kullanır. Bulunduğunda effective değer
  `movement_speed` range'i `0..1024` içinde hesaplanır ve Java `double` →
  `float` daraltmasıyla `Player.getSpeed()` sonucuna çevrilir.
- Ground friction `0.6` veya altındaysa hız doğrudan kullanılır; daha büyük
  friction değerinde Java'nın `0.21600002F / friction^3` katsayısı uygulanır.
- Bu dilim yalnız bu attribute lookup'ını ve grounded friction tüketimini
  bağlar. Havada uçuş hızı, su/lav ve tam LocalPlayer tick akışı bu kaydın
  kapsamında değildir.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/player-attribute-movement-speed-binding` trace'i dört state
gözlemi taşır:

| Gözlem | Kabul | Player speed | Friction speed |
| --- | --- | --- | --- |
| `default-player-speed` | `true` | `3dcccccd` | `3dcccccd` |
| `modified-attribute-ground-speed` | `true` | `3f2147ae` | `3e88147b` |
| `replacement-attribute-ground-speed` | `true` | `3ecccccd` | `3ecccccd` |
| `wrong-player-preserves-speed` | `false` | `3ecccccd` | `3ecccccd` |

Java referans kimliği:
`37ddf0419916b3d4560b7abeb8f362c670158f3bdd13b7486e7b18c5a4856887`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2973** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`ClientPlayerState::movement_speed`, pinned Player attribute ID `26` için
effective değeri Java range'iyle çözer ve attribute yoksa `0.1F` fallback'ini
verir. `ClientWorld::apply_local_player_friction_influenced_speed_from_attributes`
bu sonucu mevcut friction helper'ına bağlar; önceden çözümlenmiş speed alan
yardımcı korunur.

Debug adapter SHA-256 değeri
`b4e8c1bea886985576fce270c5fbc423c1268d100972e2a35cfc981beff0dc8a` ile aynı
capture üzerinde **2973/2973** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`4213516e948cf0967a971962b83e4adaabb903ff0498fd322b1aa4cd8c29318f` ile aynı
capture üzerinde **2973/2973** eşleşti; fark `null` kaldı.

Çalışan kontroller:

```text
cargo test --locked --offline --quiet
189 passed; 0 failed

cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --all -- --check

python3 tools/behavior/run.py compare --family math \
  --jdk-home /usr/lib/jvm/java-25-openjdk \
  --adapter target/debug/examples/behavior_adapter
case_count=2973
difference=null
status=parity_passed

python3 tools/behavior/run.py compare --family math \
  --jdk-home /usr/lib/jvm/java-25-openjdk \
  --adapter target/release/examples/behavior_adapter
case_count=2973
difference=null
status=parity_passed
```

## Açık kapsam

Attribute registry'sinin tüm default state'inin ClientWorld'e taşınması,
movement speed'in su/lav/hava travel ve LocalPlayer tick akışına bağlanması,
sprint/effect/item modifier yaşam döngüsü, diğer gameplay attribute'ları,
collision-aware konum güncellemesi ve oynanabilir istemci açıktır.
