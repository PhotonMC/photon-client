# P10.g — LocalPlayer sprint trigger window — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2 LocalPlayer.aiStep içindeki non-passenger sprint
trigger window geçişini karşılaştırır. Kanıt sınıfı client-parity'dir: aynı
özgün input izleri Java oracle'ına ve production Rust ClientWorld yoluna
verilir. Sprint durdurma, collision, travel ve movement packet üretimi bu
kaydın kapsamına alınmaz.

## Java kaynak incelemesi

Pinned Java Edition 26.2 / protokol 776 / Java 25 kaynaklarında şu yollar
izlendi:

- client/player/LocalPlayer.java:768-769, pozitif sprintTriggerTime
  değerinin her aiStep başında bir azaltıldığını gösterir.
- client/player/LocalPlayer.java:777-779, input tick'inden önceki shift ve
  forward state'inin yeni input'tan ayrıldığını gösterir.
- client/player/LocalPlayer.java:800-817, önceki shift, sprint'e izin vermeyen
  item kullanımı veya yeni backward input'ının pencereyi temizlediğini; uygun
  sprint predicate'i sonrasında önceki forward yoksa pencere açma ya da ikinci
  forward ile sprint başlatma dalını ve sprint tuşunun doğrudan başlatma
  yolunu tanımlar.
- client/player/ClientInput.java:5-27, keyPresses, moveVector, tick() ve
  hasForwardImpulse() alanlarının bu geçişteki rolünü tanımlar.
- client/player/LocalPlayer.java:1141-1147, mevcut sprint state'i, güncel
  forward impulse ve non-passenger sprint koşullarının ortak başlangıç kapısını
  oluşturduğunu gösterir.

Java oracle, constructor çalıştırılmadan ayrılan LocalPlayer fixture'ında
gerçek private canStartSprinting metodunu reflection ile çağırır. Fixture
ClientInput.tick() geçişinde önceki key/vector snapshot'ını korur; trigger
state'i için gerçek aiStep içindeki sıralı dallar kontrollü olarak yürütülür.
Renderer, portal, collision ve ağ yan etkileri fixture'a alınmaz.

## Yeni dal ve varyant kararı

- İlk kez çalışan Java dalı, sprintTriggerTime azaltma, önceki input ile
  güncel input ayrımı ve window içindeki ikinci forward input'ının sprint
  transition'ına bağlanmasıdır.
- Öncekilerden farklı gözlem, aynı sequence içinde ilk forward'ın pencereyi
  açması, release sırasında pencerenin azalması, ikinci forward'ın pencere
  içindeyse sprint başlatması ve pencere dışı/shift/backward input'ının state'i
  temizlemesidir.
- Aynı sonucu veren mevcut kayıtların sayısı **0**'dır. P10.c yalnız başlangıç
  predicate'ini, P10.f.2 ise doğrudan sprint setter/attribute transition'ını
  ölçer; bu kayıt ikisini önceki ve güncel input state'iyle birleştirir.

## Davranış sözleşmesi

- Her non-passenger tick'te pozitif trigger zamanı bir azalır.
- Trigger dalı, önceki input snapshot'ındaki shift ve forward bilgisini güncel
  input snapshot'ından önce okur; ClientInput.tick() sonrasında current
  backward bilgisini değerlendirir.
- Önceki shift, sprint'i yavaşlatan item kullanımı veya current backward
  input'ı trigger zamanını sıfırlar.
- Sprint başlangıç predicate'i geçerse ve önceki tick'te forward impulse yoksa,
  pozitif pencere ikinci forward input'ında sprint başlatır; pencere yoksa
  sprint_window değeriyle yeni pencere açılır.
- Current sprint key, predicate açıkken sprint'i doğrudan başlatır.
- Bu dilim sprinting state'i zaten true olduğunda durdurma dallarını ve
  passenger vehicle capability'sini çalıştırmaz.

## Test girdileri ve Java yakalama

tools/behavior/cases.py içindeki tek parametrik
client-world/player-sprint-trigger-window kaydı dört bağımsız sequence taşır:

| Sequence | Gözlenen geçiş |
| --- | --- |
| double-tap-within-window | 7 → 6 → 5; ikinci forward'da sprint true |
| window-expiry-and-sprint-key | yedi boş tick sonrası 0; sonraki forward yeni pencere açar, sprint key hemen başlatır |
| shift-resets-window | current shift sonraki tick'te pencereyi 0 yapar |
| backward-resets-window | current backward pencereyi 0 yapar |

Java referans kimliği:
f20caaa89dfeeb541cb9560a736a3eb11853bcae906f41d5b15d3ef552a673b0

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2979** kayıt
olarak doğrulandı ve fark null kaldı.

## Rust doğrulaması

crates/photon-world/src/input.rs içindeki advance_local_player_sprint_trigger, Java'nın
previous/current input sırasını ve non-passenger başlangıç koşullarını saf
state geçişi olarak uygular. ClientPlayerState, bir sonraki çağrının
previous snapshot'ı için keyPresses, move vector ve trigger zamanını saklar.
ClientWorld::apply_local_player_sprint_trigger, state değiştiğinde mevcut
apply_sprinting yolunu çağırarak shared flag ve movement speed modifier
bağını korur.

Debug adapter SHA-256 değeri
97a8823f8b2b32ce19217edde3cb725aa5dbf9443da399659de430e36a90e3d9 ile aynı
capture üzerinde **2979/2979** eşleşti; fark null kaldı.

Release adapter SHA-256 değeri
8b742eadff36a917cda8c8b0be40fc4ff12311d9f61feb514c99be62aa3b51da ile aynı
capture üzerinde **2979/2979** eşleşti; fark null kaldı.

Yeni davranış için
world::tests::local_player_sprint_trigger_tracks_previous_input_and_window
testi, gerçek ClientWorld local player state'inde önceki input snapshot'ı,
window azalmasını, double-tap sprint transition'ını ve attribute etkisini
raw f32 bitleriyle doğrular.

Çalışan kontroller:

    python3 tools/behavior/run.py capture --family math --jdk-home /usr/lib/jvm/java-25-openjdk
    python3 tools/behavior/run.py verify --family math --jdk-home /usr/lib/jvm/java-25-openjdk
    cargo build --locked --offline --example behavior_adapter
    python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/debug/examples/behavior_adapter
    cargo build --locked --offline --release --example behavior_adapter
    python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/release/examples/behavior_adapter
    cargo test --locked --offline local_player_sprint_trigger_tracks_previous_input_and_window
    cargo test --locked --offline --quiet
    cargo clippy --locked --offline --all-targets -- -D warnings
    cargo fmt --all -- --check
    git diff --check
    python3 tools/behavior/test_runner.py

## Vanilla server-conformance ayrımı

Bu dilim yalnız local client input state'ini güncellediği ve serverbound sprint
ya da movement paketi üretmediği için yeni bir server-conformance sonucu
çıkarmaz. Kullanıcının belirlediği baseline ile resmi Mojang vanilla 26.2
sunucusuna karşı connection, runtime world construction ve movement probe'ları
başarıyla çalıştırıldı; bu wire kabulü [oyun bağlantısı kaydında](game-connection-conformance.md)
ve [runtime kaydında](client-runtime-conformance.md) tutulur. Pumpkin bu
dilimde referans sunucu olarak kullanılmadı.

## Açık kapsam

Gerçek window option lookup'ı, tam LocalPlayer.aiStep sırası, sprint
durdurma/underwater yüzme dalları, auto-jump ve flight geçişleri,
collision-aware movement, travel/fizik, movement packet eşikleri, passenger
vehicle sprint capability'si ve görsel oynanabilir istemci açıktır.
