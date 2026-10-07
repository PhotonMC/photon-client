# P09.k — ClientWorld world-border state — [client-parity] birleşik kayıt

Bu kayıt, vanilla istemcinin world-border packet güncellemelerini sorgulanabilir
`ClientWorld` state'ine uygulamasını taşır. Kanıt sınıfı `client-parity`'dir:
aynı initialize, ayrı güncelleme ve tick olayları Java oracle'ında ve Rust
`ClientWorld` adapter'ında çalıştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `client/multiplayer/ClientLevel.java:172,778-780`, her `ClientLevel` kendi
  `WorldBorder` nesnesini taşır ve `getWorldBorder` ile döndürür.
- `client/multiplayer/ClientPacketListener.java:1875-1890`, initialize-border
  handler center'ı uygular; lerp süresi pozitifse old/new size arasında lerp
  başlatır, değilse new size'ı statik boyut olarak set eder; ardından
  absolute-max ve warning alanlarını yazar.
- `client/multiplayer/ClientPacketListener.java:1892-1918`, center, lerp size,
  statik size, warning blocks ve warning time packet'leri doğrudan ilgili
  `WorldBorder` setter'larına yönlendirilir.
- `client/multiplayer/ClientLevel.java:296-303,436-444`, normal client tick'inde
  border tick'i game time artırılmadan önce çalışır.
- `world/level/border/WorldBorder.java:40-45,155-201,216-283`, default state,
  center/size/lerp metadata'sı, absolute-max ve warning alanları ile extent
  geçişleri tanımlanır.

## Sözleşme

- Yeni client world border'ı center `(0, 0)`, Java float sabitinden genişleyen
  default maximum size, absolute max `29999984`, warning blocks `5` ve warning
  time `15` ile başlar.
- Initialize-border olayında çağrı sırası center, `lerpTime > 0` için lerp
  veya aksi hâlde statik size, absolute-max, warning blocks ve warning time'dır.
- Ayrı center, lerp size, statik size ve iki warning olayları yalnız kendi
  world-border alanlarını değiştirir; lerp süresi ve status state içinde
  sorgulanabilir kalır.
- World tick border extent'ini önce ilerletir, sonra game time'ı bir artırır.
  Lerp tamamlandığında border statik hedef extent'ini döndürür.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik kayıt iki olay akışı taşır:

- moving initialize, iki tick, center değişimi, yeni lerp, tamamlanma ve warning
  güncellemeleri;
- statik initialize, size/center güncellemeleri, eşit hedefli lerp ve warning
  sıfırlamaları.

Java oracle, packet listener'ın çağrı sırasını gerçek Java `WorldBorder`
setter/getter ve `tick` çağrılarıyla yürütür. Rust adapter aynı olayları gerçek
`ClientWorld` world-border state'ine uygular; sonuçta center, size, edge
örnekleri, absolute-max, warning, lerp metadata'sı, status, shape ve game time
karşılaştırılır. Beklenen sonuç Rust çıktısından türetilmez.

## Rust doğrulaması

`ClientWorld` artık kendi `ClientWorldBorder` alanını taşır. Initialize ve ayrı
packet güncelleme yönlendirmeleri üretim API'sinde tutulur; `ClientWorld::tick`
Java `ClientLevel.tick` sırasındaki border güncellemesini uygular. P09.i ve
P09.j collision shape/extent yordamları bu state üzerinden yeniden
çağrılabilir.

Java referans kimliği:
`2c0e64b14dd9b81db896f55a5e9d11f04cc25a404e4a5681447724ae5126113f`.

Math corpus'undaki 2922 kayıt, Java'nın iki ardışık capture çıktısında kararlı
kaldı. Java `verify` başarılı oldu. Rust debug adapter SHA-256 değeri
`5ef7582966964cf68e75f7ae0a7df659fe29195b5b34106f17da045e336a30e5`, release
adapter SHA-256 değeri
`88321be23b7d3e49f4f0e0835b7aabcbfa5a5c7cc5eb7f4904daca067f9e3b1e` ile iki
karşılaştırma da **2922/2922** eşleşti.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

## Açık kapsam

World-border packet state'i gerçek `ClientWorld` içine alındı. Block/entity/
world-border shape'lerinin tek `noCollision` sorgusunda birleşmesi, block-state
registry/context çözümü, canlı entity collider state'i, border distance/warning
render/damage/teleport ve player movement etkileri sonraki P09/P10/P14
dilimleridir. Bu kayıt canlı sunucu kabulü değildir.
