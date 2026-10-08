# P09.ac — Entity step-up candidate movement selection — [client-parity] birleşik kayıt

Bu kayıt, Java `Entity.collide` içindeki step-up adaylarının çözülen hareketle
karşılaştırılmasını ve ilk uygun adayın seçilmesini Rust world yardımcısıyla
karşılaştırır. Kanıt sınıfı `client-parity`'dir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/entity/Entity.java:1142-1170`, normal `movementStep` sonrası step-up
  kapısını, grounded kutuyu, aday listesini ve ilk strict yatay mesafe seçimini
  belirler.
- `world/entity/Entity.java:1173-1193`, aday yüksekliklerini üretir ve sıralar;
  bu ortak helper P09.c kaydında ayrıca doğrulanmıştır.
- `world/entity/Entity.java:1245-1260`, her aday için eksen sırasına göre
  `collideWithShapes` hareketini çözer.
- `world/phys/Vec3.java:188-194`, karşılaştırılan yatay mesafe karesini yalnızca
  X ve Z çarpımlarının kaynak sırasındaki toplamı olarak hesaplar.

## Davranış sözleşmesi

- Aday yükseklikleri grounded kutunun minimum Y'sine göre, P09.c'deki Java
  filtresi ve sıralamasıyla alınır.
- Her aday için `(movement.x, candidate, movement.z)` vektörü grounded kutu ve
  aynı collider listesiyle Java `collideWithShapes` yolundan çözülür.
- Çözülen adayın yatay mesafe karesi `movementStep` yatay mesafe karesinden
  **strict büyükse**, sıralı listedeki ilk aday seçilir ve döngü biter.
- Kazanan sonuçtan `boundingBox.minY - groundedBox.minY` değeri yalnızca Y
  ekseninde çıkarılır. Hiçbir aday strict olarak ilerlemiyorsa `movementStep`
  aynen döner.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`shape/step-up-selection` kaydı beş özgün gözlem taşır:

| Gözlem | Java seçimi |
| --- | --- |
| low-obstacle | `(2, 0.5, 0)` |
| first-better-candidate-wins | `(1.5, 0.25, 0)`; sonraki daha uzun aday denenmeden döner |
| equal-horizontal-keeps-step | `(2, 0, 0)`; eşitlik strict `>` ile reddedilir |
| ground-offset | `(2, 0, 0)`; kazananın Y'sinden zemin farkı çıkarılır |
| no-candidate-falls-back | `(0, 0, 0)`; `movementStep` korunur |

Java oracle, pinned istemci sınıfındaki gerçek `Entity.collectCandidateStepUpHeights`
ve `Entity.collideWithShapes` private metotlarını reflection ile çağırır; seçim
döngüsü kaynakta gözlenen `Entity.collide` sırasını kontrollü pre/post dünya
durumu üzerinde yürütür. Beklenen sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`c68fbc13eecb83a0d90f85f8cf7f0638e9e9a40af01e3910c2363e753d15eef5`.

Math corpus'u iki Java capture çalışmasında kararlı kaldı; toplam 2944 kayıt
karşılaştırıldı.

## Rust doğrulaması

`select_step_up_movement`, P09.c aday helper'ını, mevcut Java uyumlu eksen
çözümünü, strict yatay mesafe karşılaştırmasını, ilk kazanan erken dönüşünü ve
grounded/original kutu Y farkı düzeltmesini tek parametreli çağrıda birleştirir.

Debug adapter SHA-256 değeri
`9bfb0b7019b0a3ee4e89a4305279e112ca1d2a3761b5a5928b9e845e34bb08c3` ile
**2944/2944** eşleşti; fark `null` kaldı.
Release adapter SHA-256 değeri
`12ed9531224ca257d0aebeb061ccc937fcee6e23eac70376125c6fe2edb927fc` ile aynı
capture üzerinde **2944/2944** eşleşti; fark `null` kaldı.

Çalıştırılan dilim kontrolleri:

```text
python3 tools/behavior/run.py capture --family math --jdk-home /tmp/photon-jdk-wrapper
cargo test --locked --offline step_up_selection_uses_first_strictly_longer_candidate
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /tmp/photon-jdk-wrapper --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /tmp/photon-jdk-wrapper --adapter target/release/examples/behavior_adapter
```

## Açık kapsam

Bu kayıt `Entity.collide` içindeki step-up kapısının gerçek `Level` collider
toplama çağrılarını, `maxUpStep` attribute/Player override'ını, dünya-border
filtrelemesini ve `move` sonrası collision state yazımını kapsamaz. Bunlar
P09.a–P09.ab kayıtlarında kısmen ölçülen veya roadmap'te açık kalan bağımlılıklar
olarak kalır; P10 yerel hareketi için gerçek oyuncu tick/input bağlantısı ayrıca
gereklidir.
