# P08.c-light.1 — ClientWorld ışık kuyruğu ve section yayını — [client-parity]

Bu kayıt, yüklü olmayan bir chunk için gelen sky/block ışık section'larının
istemci world kopyasında bekletilmesini, chunk kurulduğunda yayınlanmasını ve
yüklü chunk'ta yeni layer ile değiştirilmesini kapsar. Kanıt sınıfı
`client-parity`'dir: aynı özgün sentetik trace, Java 26.2'nin gerçek
`LevelLightEngine` yoluna ve production Rust `ClientWorld` yoluna verilir.

Önceki [P08.c-light server-conformance kaydı](client-world-light-conformance.md)
vanilla sunucusunun wire kabulini gösterir. Bu kayıt onun yerine geçmez ve
sunucu kabulini istemci eşitliği kanıtı olarak kullanmaz.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu dallar incelendi:

- `client/multiplayer/ClientPacketListener.java:850-862,2385-2405`, chunk ile
  gelen ve bağımsız gelen ışık güncellemelerini client level ışık kuyruğuna
  bırakır; `:2537-2558` maskelenmiş section listesini okur.
- `client/multiplayer/ClientLevel.java:274-290,493-500`, kuyrukları FIFO
  olarak çalıştırır ve ardından light engine güncellemelerini yürütür.
- `world/level/lighting/LevelLightEngine.java:118-166`, layer verisini
  section konumuna kuyruğa alır, görünür section verisini ayırır ve level
  yüksekliğinden light section aralığını üretir.
- `world/level/lighting/LayerLightSectionStorage.java:29-32,70-73,178-214`,
  görünür olmayan section verisini `queuedSections` içinde tutar; section
  durumu güncellenince layer verisini görünür depoya alır.
- `world/level/chunk/DataLayer.java:22-70`, 2048 byte'lık layer'da her byte'ın
  düşük ve yüksek nibble'ını Java'nın Y/Z/X yerel sırasıyla çözer.

`tests/behavior/java/MathOracle.java` içindeki fixture bu sınıfların gerçek
metotlarını çağırır. Fixture'ın `LightChunkGetter` kısmı propagation için
chunk üretmez; böylece bu trace, paket layer'larının kuyruklanması ve section
yayını dalını izole eder.

## Yeni dal ve varyant kararı

Bu dilim, daha önce yalnız wire veya loaded-light state'i ölçen kayıtlardan
ayrı bir Java branch'i ilk kez çalıştırır: light layer, hedef chunk görünür
olmadan önce queue state'ine yazılır ve section status sonradan görünür hale
gelir. Öncekilerden farklı gözlem, iki ardışık unloaded update'in son layer'ı
yayınlaması, data/empty maskelerinin iki section olarak sayılması ve empty
section'ın sıfır nibble üretmesidir. Bu dal için aynı sonucu veren mevcut
`client-parity` kayıtlarının sayısı **0**'dır; eski light queue kaydı
`server-conformance` olduğu için bu sayıya dahil değildir.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25`'tir. Dimension bounds
  `min_y=-64`, `height=384` olarak sabitlenir. Bu durumda level section sayısı
  24, light section sayısı 26 ve light minimum section Y'si `-5`'tir.
- Her layer'da data maskesi ve empty maskesi birlikte taranır. Data biti varsa
  listedeki 2048 byte kullanılır; yalnız empty biti varsa sıfır `DataLayer`
  yayınlanır. Data ve empty aynı bit için verilirse data önceliklidir.
- Chunk yüklenmeden gelen ilk iki güncelleme FIFO bekleyen state olarak
  tutulur. Aynı chunk yüklenip section durumu yayınlandığında son güncelleme
  görünür layer'a ulaşır; ilk layer'ın üzerine yazıldığı düşük/yüksek nibble
  sorgularıyla gözlenir.
- Yüklü chunk'a gelen güncelleme doğrudan mevcut layer'ı değiştirir. `0x2e`
  sky byte'ı `14/2`, `0x7f` block byte'ı `15/7` olarak düşük/yüksek nibble'a
  ayrılır. Önceki `0x3c` ve `0x4d` değerleri chunk yüklenirken sırasıyla
  `12/3` ve `13/4` olarak görünür.
- Rust tarafında bekleyen queue, `ClientWorld::replace_chunk` sırasında
  addressed chunk'a uygulanır. `LightApplication` queued update için data ve
  empty maskelerinin geçerli section sayısını raporlar; görünür sorgu yalnız
  chunk kurulduktan sonra light map'ten yapılır.

## Java yakalama

`tools/behavior/cases.py` içindeki
`client-world/light-update-queue-and-publication` trace'i şu sırayı kullanır:
aynı unloaded chunk'a iki sky/block update, tek non-empty section ile chunk
kurulumu, empty section sorguları ve loaded layer replacement. Java oracle
gerçek `LevelLightEngine.queueSectionData`, `updateSectionStatus` ve
`runLightUpdates` çağrılarını kullanır.

Java referans kimliği:
`c266843ba5cbdaaccb8de38d6170d7741c4769a986da3de6b0b29be9cd7d6b0c`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2967** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`ClientWorld::apply_light_updates` artık unloaded chunk için geçerli layer
section'larını pending queue'ya alır. `replace_chunk` hedef chunk'ın pending
güncellemelerini FIFO sırayla uygular; loaded chunk yolu aynı
`apply_light_layer` storage'ını kullanır. Adapter, layer nibble'larını ve
world light sorgularını production API üzerinden gözlemler.

Debug adapter SHA-256 değeri
`2d24e4bdf7020a126c0154427519db7a0e17d35f804bac7a3c8b6493a453bc4e` ile aynı
capture üzerinde **2967/2967** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`4a8dea955d6f667869c5e6d4c26776d2be46a909bbdca1961e9c4fde4b1c80b0` ile aynı
capture üzerinde **2967/2967** eşleşti; fark `null` kaldı.

```text
cargo test --locked --offline --quiet
188 passed; 0 failed

python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/debug/examples/behavior_adapter
case_count=2967
difference=null
status=parity_passed

cargo build --locked --offline --release --example behavior_adapter

python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/voxel-client/target/release/examples/behavior_adapter
case_count=2967
difference=null
status=parity_passed
```

## Açık kapsam

Bu kayıt `ClientLevel` ana döngüsünün gerçek frame görev kuyruğunu, komşu
section propagation/relight hesaplarını, light column enable/disable
semantiğini, unload sırasında queued layer temizliğini, sky default brightness
ve block-emission çözümünü, section dirty/rebuild ile render/AO/tint
invalidasyonunu kapsamaz. Tam bağımsız istemci packet listener entegrasyonu,
block entity etkileri ve oynanabilir client de açıktır.

## 2026-09-29 notu

Rust tarafı artık referansın ışık motorunun portudur
([P08.c-light.2](client-light-engine-conformance.md)). Yukarıdaki "bekleyen
queue `replace_chunk` sırasında uygulanır" modeli kaldırıldı: güncellemeler
kare kuyruğuna girer ve motorun kendi kuyruğundan yayınlanır. Bu kaydın iki
vakası yeni motorla eşleşmeye devam ediyor.
