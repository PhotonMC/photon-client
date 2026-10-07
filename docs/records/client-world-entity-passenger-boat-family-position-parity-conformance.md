# P08.az.1 — ClientWorld boat family passenger geometry — [client-parity]

Bu kayıt, normal boat, chest boat, bamboo raft ve bamboo chest raft
passenger'ının Java istemcisindeki `positionRider` geometri dallarının Rust
`ClientWorld` kopyasında eşleşmesini karşılaştırır. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik input Java oracle'ına ve production
Rust adapter'a verilir.

[P08.az server-conformance kaydı](client-world-entity-passenger-boat-family-conformance.md)
vanilla sunucusuna bağlı family geometry gözlemlerini tutar. O kayıt
server-conformance kanıtıdır; bu kayıt Java istemci state'ini karşılaştırır.

## Java kaynak incelemesi

Pinned Java Edition 26.2 kaynaklarında şu çağrı zinciri izlendi:

- `world/entity/Entity.java:2380-2417`, `rideTick` sonrasında vehicle'ın
  `positionRider` yolunu çağırır. Boat passenger riding position'ından
  passenger'ın vehicle attachment noktası çıkarılır.
- `world/entity/EntityAttachments.java:47-70`, attachment noktası olmayan
  snowball için feet fallback'inin sıfır olduğunu gösterir.
- `world/entity/vehicle/boat/AbstractBoat.java:132-150`, tek passenger'da
  concrete boat sınıfının ride height'ını ve
  `getSinglePassengerXOffset()` değerini birleştirir; offset vehicle yaw'ıyla
  döndürülür.
- `world/entity/vehicle/boat/AbstractChestBoat.java:43-50`, chest family için
  tek passenger z-offset'ini `0.15F` yapar ve kapasiteyi `1` ile sınırlar.
- `world/entity/vehicle/boat/Boat.java:15-17` ve
  `world/entity/vehicle/boat/ChestBoat.java:15-17`, ride height'ı
  `0.5625F / 3.0F` olarak hesaplar. `Raft.java:15-17` ve
  `ChestRaft.java:15-17`, aynı dimensions yüksekliğini `0.8888889F` ile
  çarpar.

Oracle fixture'ı normal `Boat` yanında gerçek vanilla `ChestBoat`, `Raft` ve
`ChestRaft` sınıflarını Unsafe ile allocate eder. Entity type, dimensions,
position, passenger listesi ve transform alanları Java sınıfının gerçek
`positionRider` çağrısını çalıştıracak şekilde başlatılır; fixture `tick()`
gövdesi no-op bırakılarak bu dilimde yalnız `ClientLevel` tick sırası ve seat
geometrisi ölçülür.

## Yeni dal ve varyant kararı

Bu kayıt, P08.ax.1'de yalnız normal oak boat için karşılaştırılan seat yolunda
ilk kez iki Java dalını çalıştırır: `AbstractChestBoat` içindeki
`getSinglePassengerXOffset()` override'ı ve `Raft`/`ChestRaft` içindeki farklı
`rideHeight()` override'ları. Farklı gözlem chest family'de yaw ile dönen
`0.15F` offset'in görünmesi ve raft family'de seat y değerinin `0.5` olmasıdır.
Mevcut aynı sonucu veren `client-parity` kayıtlarının sayısı **0**'dır.

Yalnız sabiti değişen 20 entity type kopyası açılmadı. Tek parametrik corpus
kaydı, dört family dalını şu temsilci satırlarıyla kapsar: normal oak boat,
oak chest boat, bamboo raft ve bamboo chest raft.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776` ve Java `25`'tir. Loaded empty
  section içinde vehicle konumu `(2,10,2)`, vehicle yaw `90.0F`, snowball
  konumu `(2,12,2)`, movement `(0.125,0,0)` ve passenger yaw/pitch
  `10.0F/-15.0F` kullanılır. Bütün entity'lerde `NoGravity=false`'dur.
- Her satırda world tick'i önce vehicle için old transform ve tick count'u
  ilerletir, ardından snowball `rideTick` hareketini işler ve gerçek
  `vehicle.positionRider(passenger)` çağrısını yapar.
- Snowball vehicle attachment'ı `(0,0,0)` olduğundan current passenger
  konumu aşağıdaki seat geometry sonucudur. Snowball'ın tick sonrası motion'ı
  `y=-0.029700000286102295`, previous position'ı `(2,12,2)` olarak state'e
  dahil edilir.

| Family | Vehicle type | Ride height | Single passenger offset | Passenger current position |
| --- | ---: | ---: | ---: | --- |
| Oak boat | `89` | `0.1875` | `0` | `(2,10.1875,2)` |
| Oak chest boat | `90` | `0.1875` | `0.15F` → `0.15000000596046448` | `(1.8499999940395355,10.1875,2)` |
| Bamboo raft | `9` | `0.5` | `0` | `(2,10.5,2)` |
| Bamboo chest raft | `8` | `0.5` | `0.15F` → `0.15000000596046448` | `(1.8499999940395355,10.5,2)` |

## Java yakalama

`tools/behavior/cases.py` içindeki tek
`client-world/entity-passenger-boat-family-position` kaydı dört family
satırını aynı initial ve `family-seat` state'leri içinde yürütür. Java oracle
iki çalıştırmada kararlı kaldı; tekrar farkı `null`'dır.

Math capture kimliği:
`5460dd741c2f035f045143d9c7b4301ab3014d12f3d6e25cdf7b8fd1e61e6d7c`

Bu capture math ailesinde **3009** case içerir. Yeni kaydın Java çıktısında
normal/chest boat passenger y değeri `10.1875`, raft/chest raft passenger y
değeri `10.5` ve chest satırlarının yaw dönmüş x değeri
`1.8499999940395355`'tir.

Tam corpus Java capture kimliği:
`be7ceb039f947e2676b9fe9d464314630d46f082952d79531675b2ee34532311`

Toplam **3870** case'tir; capture iki Java tekrarında `difference=null` ile
`reference_ready` oldu.

## Rust doğrulaması

Adapter boş section'ı yükledikten sonra production
`ClientWorld::apply_entity`, `ClientWorld::apply_entity_metadata`,
`ClientWorld::apply_entity_passengers` ve `ClientWorld::tick` yollarını
kullanır. Vehicle type ID'si tabloyla eşleşen dört family satırında seat
yüksekliği, chest offset'i, yaw dönüşü, passenger previous position'ı ve
snowball ride tick motion state'i aynı çıktıda karşılaştırılır.

Rust tarafında yeni family sabitleri eklenmedi; mevcut
`VANILLA_BOAT_PASSENGER_GEOMETRY` tablosundaki 20 satırın type/height/offset
eşleşmesi doğrudan production passenger positioning yolundan yürütüldü.
Beklenen konumlar Java oracle çıktısından sonra yazıldı; Rust çıktısından
türetilmedi.

Debug adapter SHA-256:
`c89dca6079b3d2ba7553455b1acdb7dd8fb2df422658ab7e8b7d96b0e6900389`

```text
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
case_count=3009
difference=null
status=parity_passed

python3 tools/behavior/run.py compare \
  --adapter target/debug/examples/behavior_adapter
case_count=3870
difference=null
status=parity_passed
```

Release adapter SHA-256:
`05422ab0f04cba1cd1e6ce0e42b0125801b45a9050a74c187dea2d514407441e`

```text
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare \
  --adapter target/release/examples/behavior_adapter
case_count=3870
difference=null
status=parity_passed
```

`cargo test --locked --offline` sonucu **194 passed**,
`cargo clippy --locked --offline --all-targets -- -D warnings` sonucu temizdir.
`cargo fmt --all -- --check`, Python syntax check ve `git diff --check` de
geçti.

## Kanıt sınıfı sınırı

Bu kayıt `client-parity`'dir: aynı özgün input Java istemci oracle'ı ile Rust
adapter arasında karşılaştırılır. Vanilla sunucusunun chest boat ve raft
packet'lerini kabul etmesi [P08.az server-conformance kaydında](client-world-entity-passenger-boat-family-conformance.md)
tutulur ve bu state eşitliğinin yerine geçmez.

## Açık kapsam

Çoklu passenger seat sırası, chest kapasite reddi, Animal passenger offset'i,
Player ve pose/scale/dimensions'a bağlı attachment'lar, nonzero boat
`deltaRotation`, passenger rotation yan etkileri, nested controlling
passenger, collision, dismount, gerçek oynanabilir istemci bağlantısı ve
render/frame timing'i açık kalır. Bu kayıt P08'in `client-parity` kapanışı
değildir.
