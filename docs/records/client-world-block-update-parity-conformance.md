# P08.c.1 — ClientWorld block update ve section sayaçları — [client-parity]

Bu kayıt, yüklü bir `ClientWorld` section'ındaki tek block-state
değişikliğinin hücreyi ve section sayaçlarını Java Edition 26.2 ile aynı
şekilde güncellemesini kapsar. Kanıt sınıfı `client-parity`'dir: aynı özgün
section payload'ı Java section state'ine ve production Rust `ClientWorld`
yoluna verilir.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu yollar esas alındı:

- `client/multiplayer/ClientPacketListener.java:935-938`, tekil block update
  paketini `ClientLevel.setServerVerifiedBlockState` yoluna verir.
- `client/multiplayer/ClientLevel.java:199-203`, prediction handler güncellemeyi
  tüketmediğinde temel level kopyasına state yazar.
- `world/level/chunk/LevelChunk.java:268-292`, boş section'a air yazma ve aynı
  state yazma erken dönüşlerini, ardından section emptiness geçişini seçer.
- `world/level/chunk/LevelChunkSection.java:57-99`, `setBlockState` önceki ve
  yeni state'sinin air/fluid özelliklerine göre `nonEmptyBlockCount` ve
  `fluidCount` alanlarını azaltıp artırır ve önceki state'i döndürür.
- `world/level/block/Block.java:127-139`, numeric state lookup'ının ve state ID
  dönüşünün Java fallback davranışını tanımlar.

## Yeni dal ve varyant kararı

Bu dilim, P08.c'deki server-conformance packet kabulünden farklı olarak Java
istemci section state'inin mutable `setBlockState` dalını ilk kez karşılaştırır.
İlk gözlem block hücresinin değişmesiyle birlikte `hasOnlyAir`,
`hasFluid` ve iki short section sayacının değişmesidir. Önceki section-read
parity kaydı yalnız başlangıç payload'ını okur; P08.c server-conformance kaydı
Java client state'iyle karşılaştırma yapmaz. Aynı sonucu veren mevcut parity
kaydı sayısı bu dal için **0**'dır.

## Davranış sözleşmesi

- Başlangıç section'ı `ClientWorld` içine chunk koordinatı `(-2, 3)` ve
  `min_y=-64` ile yazılır. Dünya ve section koordinatlarında Java'nın
  `floorDiv`/`floorMod` semantiği korunur.
- Güncelleme hedefi yüklü chunk ve section içindeyse production world hücresi
  yeni state ID'sini döndürür; Java'daki boş section'a air ve aynı state erken
  dönüşleri `false` kabul edilir. Eksik section ve başka chunk reddedilir.
- Java'nın update sırası korunur: önceki state section sayaçlarına çıkarılır,
  yeni state sayaçlara eklenir, ardından hücre yazılır. Aynı state'e yazma
  sayaçları değiştirmez.
- Kontrollü parity girdisi Java registry'sinden doğrulanan `air=0`,
  `stone=1` ve source bucket akışında gözlenen `water=86` state ID'lerini
  kullanır. Bu üç state için `nonEmptyBlockCount`, `fluidCount`,
  `hasOnlyAir` ve `hasFluid` gözlenir.

## Java yakalama

`tools/behavior/cases.py` içindeki
`client-world/block-update-section-counters` kaydı, aynı section üzerinde
air→stone, stone→water, water→air, negatif yerel koordinat sınırı, aynı-state
yazımı, eksik section ve chunk dışı hedef adımlarını yürütür. Java oracle,
pinned client JAR'ındaki gerçek `LevelChunkSection.read` ve
`LevelChunkSection.setBlockState` yollarını çağırır; sayaçlar private field
reflection ile yalnızca gözlem için okunur.

Java referans kimliği:
`f808c572655c9aa6993f2f9e83b585e686cbd3b5e107d6f006e3bee6de065fe8`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2966** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`ClientWorld::update_block_state`, loaded section hücresini
`ClientSection::set_block_state` üzerinden değiştirir. Sayaç geçişi Java'nın
air/non-air ve water-fluid gözlemleriyle aynı production state yolunda tutulur;
adapter sonucu `block_state_at` ve `section_counts_at` üzerinden sorgular.

Debug adapter SHA-256 değeri
`8eb114fead097d53baca9a7e3fb859e8e2a647c237e7336cf32e5811948418cd` ile aynı
capture üzerinde **2966/2966** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`0cde0de3f98843e7de01792dedf5bad79597ef08e455c52d5a7b3e5c49d900d4` ile aynı
capture üzerinde **2966/2966** eşleşti; fark `null` kaldı.

```text
cargo test --locked --offline --quiet
188 passed; 0 failed

python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/debug/examples/behavior_adapter
case_count=2966
difference=null
status=parity_passed

python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/release/examples/behavior_adapter
case_count=2966
difference=null
status=parity_passed
```

## Açık kapsam

Tam block-state registry ve `Block.stateById` fallback'i, tüm fluid türleri ve
waterlogged state'ler, random-tick sayaçları, prediction handler çakışması,
light propagation/invalidation, block entity etkileri ve bağımsız
`ClientLevel` nesnesi bu dilimin dışındadır. Bu kayıt P08 bölümünü kapatmaz;
yalnız mutable block update'ın ölçülen state/counter dalını kapatır.
