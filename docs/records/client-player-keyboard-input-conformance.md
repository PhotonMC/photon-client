# P10.f.4 — KeyboardInput snapshot'ın ClientWorld LocalPlayer state'ine bağlanması — [client-parity]

Bu kayıt, Java istemcisinin KeyboardInput.tick() ile ürettiği yedi alanlı
input snapshot'ının ClientWorld içindeki local player applyInput yoluna
ulaşmasını kapsar. Kanıt sınıfı client-parity'dir: aynı özgün sentetik trace,
Java Edition 26.2 oracle'ına ve production Rust world yoluna verildi.
Bu dilim canlı sunucu kabulü veya görsel oyun oturumu değildir.

## Java kaynak incelemesi

Pinned Java Edition 26.2, protokol 776 ve Java 25 kaynaklarında şu yollar esas
alındı:

- client/player/KeyboardInput.java:7-36, Options içindeki yedi KeyMapping
  değerini ClientInput.keyPresses snapshot'ına taşır, karşıt yönleri aynı
  eksende sıfırlar ve (leftImpulse, forwardImpulse) vektörünü normalize eder.
- client/player/ClientInput.java:6-15, keyPresses ve moveVector alanlarını
  LocalPlayer tarafından tüketilen ortak input state'i olarak taşır.
- client/player/LocalPlayer.java:692-705, kontrollü kamerada
  input.getMoveVector() sonucunu modifyInput üzerinden xxa/zza alanlarına,
  input.keyPresses.jump() değerini jumping alanına yazar.
- client/player/LocalPlayer.java:707-736, sıfır olmayan input için 0.98F,
  eşya kullanım çarpanı, sneaking attribute değeri ve kare hareket
  düzeltmesinin sırasını uygular.

Java oracle, Player.createAttributes().build() ile gerçek AttributeMap
oluşturur ve bunu constructor çalıştırılmadan kurulan kontrollü LocalPlayer
fixture'ına bağlar. Her operation kendi Options/KeyMapping snapshot'ını
üretir; public KeyboardInput.tick() ve LocalPlayer.applyInput() gerçek Java
sınıflarında çağrılır.

## Yeni dal ve varyant kararı

- İlk kez çalışan Java dalı, KeyboardInput.tick() içindeki Options key
  mapping snapshot'ının ClientInput alanlarından okunup ClientWorld local
  player input state'ine bağlanmasıdır.
- Öncekilerden farklı gözlem, yedi boolean alanın aynı world trace'inde
  keyPresses olarak korunması, karşıt tuşların sıfırlanması, normalize
  vektörün xxa/zza değerlerine ulaşması, jump snapshot'ının world state'ine
  yazılması ve mevcut sneaking attribute yolundan geçmesidir.
- Aynı sonucu veren mevcut kayıtların sayısı **0**'dır. P05.c yalnızca
  KeyboardInput sonucunu bağımsız olarak ölçer; P10.b doğrudan hazırlanmış
  ClientInput ile applyInput state yazımını ölçer; P10.f.3 ise doğrudan
  Vec2 input'a attribute lookup uygular. Bu kayıt iki üretim yolunu tek
  ClientWorld local player state'inde birleştirir.

## Davranış sözleşmesi

- Her çağrı yedi key state'i Java sırasıyla forward, backward, left, right,
  jump, shift, sprint alanlarına snapshot'lar.
- Aynı eksenin iki yönü basılıysa impulse 0.0F; tek yön basılıysa 1.0F veya
  -1.0F olur. Hareket vektörü bu iki impulse ile kurulup Java Vec2.normalized()
  yolundan geçer.
- Local player ID'si eşleşen attribute snapshot'ı world player state'ine
  uygulanır. Yanlış ID attribute ve input state'ini kabul etmez; önceki
  local player attribute değeri korunur.
- Kontrollü kamera dalında normalize vektör Java LocalPlayer.modifyInput
  sırasıyla işlenir ve xxa/zza alanlarına yazılır. Jump, snapshot'taki jump
  değerinden yazılır; sıfır hareket, modifier lookup'ını ve kare hareket
  düzeltmesini atlar.
- Bu kayıt gerçek key mapping olay kuyruğunu, pencere odağını, aiStep
  zamanlamasını, collision-aware konum güncellemesini, movement paket
  seçimini ve oynanabilir istemciyi kapsamaz.

## Test girdileri ve Java yakalama

tools/behavior/cases.py içindeki tek parametrik
client-world/player-keyboard-input-binding trace'i beş ardışık state
gözlemi taşır:

| Gözlem | Kabul | Move vector x/y | World movement x/y | Jump |
| --- | --- | --- | --- | --- |
| forward-key-reaches-local-input | true | 00000000 / 3f800000 | 00000000 / 3f7ae148 | false |
| diagonal-keys-use-sneaking-attribute | true | 3f3504f3 / 3f3504f3 | 3f16872c / 3f16872c | true |
| opposing-keys-zero-movement | true | 00000000 / 00000000 | 00000000 / 00000000 | true |
| item-and-sneaking-key-order | true | bf3504f3 / 3f3504f3 | be7ae147 / 3e7ae147 | false |
| wrong-player-preserves-key-attribute | false | 00000000 / 3f800000 | 00000000 / 3efae148 | false |

Java referans kimliği:
ba589d919b8fd3b6e4ffc92661171c6b24c517c08c1f617d5396f9cf07992264.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2977** kayıt
olarak doğrulandı ve fark null kaldı.

## Rust doğrulaması

ClientWorld::apply_local_player_keyboard_input, production
KeyboardInput::new ve KeyboardInput::tick yolunu çalıştırır; elde edilen
normalize vektörü, jump snapshot'ı ve world player attribute state'i
ClientPlayerState::apply_input_from_attributes yoluna aktarır. Adapter,
aynı production world state'inden key_presses, hareket vektörü, movement ve
jump alanlarını okur.

Debug adapter SHA-256 değeri
6fa49bcb940468974ce98494d8fa577c40553785a49214ed53e2778f0cf81990 ile
aynı capture üzerinde **2977/2977** eşleşti; fark null kaldı.

Release adapter SHA-256 değeri
bf8a855268e6a4283f2fbc52d2a74d1c542e2adf0b8cdb3f2448c579dec6ec88 ile
aynı capture üzerinde **2977/2977** eşleşti; fark null kaldı.

Yeni davranış için
world::tests::local_player_keyboard_input_binds_keys_before_world_modifiers
testi, gerçek ClientWorld local player state'inde tek yön, attribute
uygulanmış diyagonal input, jump ve karşıt tuşların sıfırlanmasını raw f32
bitleriyle doğrular.

Çalışan kontroller:

    python3 tools/behavior/run.py capture --family math --jdk-home /usr/lib/jvm/java-25-openjdk
    python3 tools/behavior/run.py verify --family math --jdk-home /usr/lib/jvm/java-25-openjdk
    cargo build --locked --offline --example behavior_adapter
    python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
    cargo build --locked --offline --release --example behavior_adapter
    python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
    cargo test --locked --offline --quiet
    cargo clippy --locked --offline --all-targets -- -D warnings
    cargo fmt --all -- --check

## Açık kapsam

Gerçek pencere/input olaylarının world tick'ine bağlanması, LocalPlayer
aiStep ve sprint trigger zamanları, crouch/visual crawl üretimi,
collision-aware konum/hız entegrasyonu, movement paketleri, canlı vanilla
sunucu oturumu ve görsel oynanabilir istemci açıktır.
