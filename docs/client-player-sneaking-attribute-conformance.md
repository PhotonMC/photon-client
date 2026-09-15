# P10.f.3 — LocalPlayer sneaking speed attribute in input — [client-parity]

Bu kayıt, Java `LocalPlayer.modifyInput` içindeki yavaş hareket dalının
`Attributes.SNEAKING_SPEED` değerini local player input state'ine bağlamasını
kapsar. Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik trace, Java
Edition 26.2 oracle'ına ve production Rust `ClientWorld` yoluna verildi. Yeni
bir wire veya canlı sunucu davranışı eklenmedi.

## Java kaynak incelemesi

Pinned Java Edition 26.2, protokol 776 ve Java 25 kaynaklarında şu yollar
esas alındı:

- `client/player/LocalPlayer.java:692-705`, kontrollü kamerada
  `modifyInput` sonucunu `xxa` ve `zza` alanlarına yazar; bu dalın üst sınıf
  fallback'inden ayrıldığı yerdir.
- `client/player/LocalPlayer.java:707-736`, input uzunluğu sıfır değilse
  temel `0.98F` ve eşya kullanım çarpanlarını uygular, ardından yavaş hareket
  durumunda `Attributes.SNEAKING_SPEED` effective değerini okuyup kare
  hareket düzeltmesini çağırır.
- `world/entity/player/Player.java:206-220`, Player attribute supplier'ında
  sneaking speed instance'ını kurar.
- `world/entity/ai/attributes/Attributes.java:92-94`, `sneaking_speed`
  attribute'unun varsayılanını `0.3` ve geçerli aralığını `0..1` olarak kurar.
- `world/entity/ai/attributes/AttributeInstance.java:139-170`, effective
  değeri modifier operation sırasıyla hesaplar ve attribute range'ine göre
  sanitize eder.

Java oracle, `Player.createAttributes().build()` ile gerçek `AttributeMap`
kurar ve bunu constructor çalıştırılmadan oluşturulan kontrollü
`LocalPlayer` fixture'ına bağlar. Public `LocalPlayer.applyInput` gerçek
attribute lookup'ı üzerinden çağrılır. Pinned `sneaking_speed` registry ID'si
31 olarak gözlendi.

## Yeni dal ve varyant kararı

- İlk kez çalışan Java dalı, `isMovingSlowly()` true olduğunda
  `Attributes.SNEAKING_SPEED` holder'ından effective değerin okunup input'a
  uygulanmasıdır.
- Öncekilerden farklı gözlem, attribute snapshot'ı ile varsayılan `0.3`
  değerinin `0.75` değerine çıkması, eşya kullanımından sonra yavaş hareket
  çarpanının uygulanması, yavaş hareket kapalıyken attribute'un input'u
  değiştirmemesi ve yanlış oyuncu ID'sinin önceki değeri korumasıdır.
- Aynı sonucu veren mevcut kayıtların sayısı **0**'dır. P10.a yalnızca
  sneaking factor'ını doğrudan modifier parametresi olarak verir; P10.b ise
  `applyInput` state yazımını default factor ile çalıştırır. Bu kayıt gerçek
  Player attribute map lookup'ını bu input dalına bağlar.

## Davranış sözleşmesi

- Local player ID'si `42` ile eşleşen attribute snapshot'ı `sneaking_speed`
  instance'ının base ve modifier state'ini günceller. Eşleşmeyen ID local
  player state'ini değiştirmez.
- Attribute bulunmadığında Java Player supplier'ının effective değeri `0.3`
  kullanılır. Bulunduğunda effective değer `0..1` aralığında Java operation
  sırasıyla hesaplanır ve `double` değer input çarpanına geçirilirken f32'ye
  daraltılır.
- `isMovingSlowly()` true, kamera kontrollü ve input uzunluğu sıfır değilse
  bu değer `0.98F` ve eşya kullanım çarpanından sonra uygulanır. Yavaş hareket
  kapalıysa attribute lookup'ı input sonucunu değiştirmez; sıfır input da
  olduğu gibi korunur.
- Bu dilim attribute snapshot'ının input dönüşümüne bağlanmasını kapsar.
  Gerçek keyboard snapshot'ı, crouch/visual crawl üretimi, LocalPlayer tick'i,
  movement efficiency ve effect/item attribute yaşam döngüsü, collision-aware
  konum güncellemesi ve oynanabilir istemci bu kaydın dışındadır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik
`client-world/player-sneaking-speed-input-binding` trace'i beş ardışık state
gözlemi taşır:

| Gözlem | Kabul | Effective sneaking speed | Input x | Input y |
| --- | --- | --- | --- | --- |
| `default-sneaking-speed` | `true` | `3fd3333333333333` | `00000000` | `3e96872c` |
| `custom-sneaking-speed` | `true` | `3fe8000000000000` | `00000000` | `3f3c28f6` |
| `item-and-sneaking-attribute-order` | `true` | `3fe0000000000000` | `3eb16622` | `3eb16622` |
| `non-slow-input-ignores-sneaking-speed` | `true` | `3feccccccccccccd` | `00000000` | `3f7ae148` |
| `wrong-player-preserves-sneaking-speed` | `false` | `3feccccccccccccd` | `00000000` | `3f61cac0` |

Java referans kimliği:
`77234ea7719ff8c91bc9aec9bda058a9197f9d9a77d34d20e73f3488ab7fb109`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2976** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`ClientWorld::apply_local_player_input_from_attributes`, eşleşen local player
attribute state'inden sneaking speed effective değerini çözer ve gerçek
`applyInput` state yazımına verir. `ClientPlayerState` aynı attribute range ve
modifier operation sırasını kullanır; attribute yoksa Java Player default
değerine döner. Adapter, input/bob/jump çıktısını production world state'inden
okur.

Debug adapter SHA-256 değeri
`fd801bbaae592ee719682b132f3748b9ff7423f0adfd84be20113c70c3d6bb25` ile aynı
capture üzerinde **2976/2976** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`2049feacda6e511f443a3221d8c2d85ccf6d0ae8abaef7884beb196a4baa3324` ile aynı
capture üzerinde **2976/2976** eşleşti; fark `null` kaldı.

Çalışan kontroller:

```text
python3 tools/behavior/run.py capture --family math --jdk-home /usr/lib/jvm/java-25-openjdk
python3 tools/behavior/run.py verify --family math --jdk-home /usr/lib/jvm/java-25-openjdk
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/release/examples/behavior_adapter
cargo test --locked --offline --quiet
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --all -- --check
```

`cargo test` sonucu **192 passed; 0 failed** oldu; yeni davranış için
`world::tests::local_player_input_reads_sneaking_speed_attribute` testi de bu
koşunun içindedir.

## Açık kapsam

Gerçek keyboard snapshot'ının tick'e bağlanması, crouch/visual crawl state'i,
`aiStep` ve sprint trigger zamanları, block/fluid lookup, movement efficiency,
effect/item attribute modifier yaşam döngüsü, collision-aware movement,
serverbound movement paketleri ve oynanabilir istemci açıktır.
