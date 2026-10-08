# P10.c — LocalPlayer canStartSprinting — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 `LocalPlayer.canStartSprinting` koşulunu
non-passenger bir local player snapshot'ı üzerinde karşılaştırır. Kanıt sınıfı
`client-parity`'dir: gerçek Java private metoduna kontrollü fixture ile verilen
state, Rust production predicate'iyle karşılaştırılır. Bu dilim sprint flag'ini
değiştiren tick geçişini veya hareket paketini çalıştırmaz.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `client/player/LocalPlayer.java:1135-1147`, sprint başlatmanın önce mevcut
  sprint state'ini, ileri impulsu ve `isSprintingPossible` sonucunu; sonra item
  kullanımı, fall-flying ve yavaş hareket kapılarını kontrol ettiğini gösterir.
- `client/player/LocalPlayer.java:563-568`, item kullanımının sprint'e izin
  vermeyen effect durumunu ve hız davranışından ayrı kapıyı taşır.
- `client/player/ClientInput.java:5-27`, ileri impulsu `moveVector.y >
  1.0E-5F` karşılaştırmasıyla belirler.
- `world/entity/player/Player.java:1592-1594`, non-passenger sprint için
  yeterli food veya `Abilities.mayfly` koşulunu sağlar.
- `client/player/LocalPlayer.java:806-817`, bu predicate'i `aiStep` içinde
  sprint trigger window ve sprint tuşu geçişleriyle birlikte kullanır; window
  state'i bu kaydın dışındadır.

## Davranış sözleşmesi

- Zaten sprinting olan oyuncu, ileri impulsu olmayan oyuncu veya impulsu
  `1.0E-5F` eşiğine eşit/küçük olan oyuncu sprint başlatamaz.
- Non-passenger durumda mobility restriction yoksa ve food yeterliyse ya da
  `mayfly` açıksa sprint mümkün olabilir. Flying, shallow-water kapısını açar.
- Sprint'e izin vermeyen item effect'i, su altında olmayan fall-flying ve su
  altında olmayan moving-slowly durumları sprint başlangıcını reddeder.
- Tüm koşullar strict boolean kapılardır; forward impulse ham f32 değeriyle
  karşılaştırılır. Epsilon veya davranış yumuşatması kullanılmaz.
- Bu kayıt `isSprinting` flag'ini set etmez. Java `aiStep` içindeki trigger
  counter, sprint start/stop transition'ı ve passenger vehicle capability'si
  sonraki dilimlerdir.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik `player-sprint/can-start`
kaydı on dört gözlem taşır:

| Gözlem | İlk kez sınanan dal |
| --- | --- |
| `eligible-ground-forward` | Tüm sprint başlangıç kapılarının açık yolu |
| `already-sprinting-blocks-start` | Mevcut sprint state'i |
| `forward-threshold-rejects-zero` | Sıfır ileri impulsu |
| `forward-threshold-rejects-small` | `1.0E-5F` strict eşiği |
| `mobility-restricted-blocks-start` | Mobility restriction kapısı |
| `insufficient-food-blocks-start` | Yetersiz food yolu |
| `mayfly-allows-insufficient-food` | `mayfly` food bypass'ı |
| `shallow-water-blocks-ground-start` | Flying olmayan shallow-water yolu |
| `flying-allows-shallow-water` | Flying shallow-water bypass'ı |
| `item-use-blocks-sprint` | Sprint'e izin vermeyen item effect'i |
| `fall-flying-blocks-out-of-water` | Suyun dışındaki fall-flying yolu |
| `fall-flying-allows-underwater` | Underwater fall-flying bypass'ı |
| `moving-slowly-blocks-out-of-water` | Suyun dışındaki yavaş hareket yolu |
| `moving-slowly-allows-underwater` | Underwater yavaş hareket bypass'ı |

Java oracle, pinned JAR içindeki gerçek private `canStartSprinting` metodunu
constructor çalıştırmadan ayrılmış fixture ve reflection ile çağırır. Fixture
gerçek `FoodData`, `Abilities` ve `ClientInput` alanlarını kurar; item-use
satırı sprint'e izin vermeyen gerçek `UseEffects` kaydıyla çalışır. Beklenen
sonuç Rust çıktısından türetilmez.

Java referans kimliği:
`b8c48ad336d8bbf7a9ccf78c8f0f9966c542e088b35d61441ceb6abea570c273`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2952** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/photon-world/src/input.rs` içindeki `can_start_local_player_sprinting`, Java predicate'inin
non-passenger koşullarını `LocalPlayerSprintGateInput` snapshot'ı üzerinden
uygular. Behavior adapter aynı production predicate'ini çağırır; Java'nın
private metodunu veya beklenen sonucu adapter içinde kopyalamaz.

Debug adapter SHA-256 değeri
`f1342aff595839f40691b0015da21a74cf2450df246c7944fb420fc639232d72` ile
**2952/2952** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`e50ef2a9876b0bf3e0473a584f3ae794a789b014ea2854fed205f54543e9189c` ile
aynı capture üzerinde **2952/2952** eşleşti; fark `null` kaldı.

Dilimde kullanılan temel kontroller:

```text
python3 tools/behavior/run.py capture --family math --jdk-home /usr/lib/jvm/java-25-openjdk
python3 tools/behavior/run.py verify --family math --jdk-home /usr/lib/jvm/java-25-openjdk
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/release/examples/behavior_adapter
```

## Vanilla server-conformance ayrımı

Bu predicate henüz sprint flag'i veya serverbound packet üretmediği için P10.c
için yeni canlı sunucu koşusu yapılmaz. Resmi Mojang vanilla 26.2 sunucusuna
karşı login ve movement wire adjunct'ı [P10.a kaydında](client-player-input-conformance.md)
tutulur. O kayıt `server-conformance` wire kabulünü gösterir; bu belge sprint
başlangıç koşulunun Java istemcisiyle eşitliğini yalnızca `client-parity`
capture'ı ile iddia eder.

## Açık kapsam

`aiStep` içindeki sprint trigger counter/double-tap sırası, sprint tuşu,
`shouldStopRunSprinting` ve `shouldStopSwimSprinting`, sprint flag/attribute
güncellemesi, passenger vehicle sprint capability'si, gerçek input tick'i,
collision, su/lav/merdiven, hareket fiziği ve serverbound sprint/movement
paketleri P10.d–e, P11 ve ilgili sonraki dilimlerde açık kalır.
