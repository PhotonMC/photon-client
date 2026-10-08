# P08.c-attribute.1 — [client-parity] LocalPlayer attribute snapshot binding

Bu kayıt, Java istemci listener'ının local player attribute snapshot'ını
world kopyasına uygulamasını ve aynı state'ten etkili değeri sorgulamasını
kapsar. Kanıt sınıfı `client-parity`'dir: aynı özgün sentetik trace, Java
26.2'nin gerçek `AttributeMap`/`AttributeInstance` yollarına ve production
Rust `ClientWorld` yoluna verilir.

[P08.j server-conformance kaydı](client-world-player-attributes-conformance.md)
packet wire alanlarını ve respawn taşımasını, [P08.p
kaydı](client-world-player-attribute-effective-conformance.md) ise vanilla
sunucudan gelen tek bir effective attribute hesabını gösterir. Bu kayıt bu
iki sonucu istemci world state'ine bağlar.

## Java kaynak incelemesi

Pinned Java 26.2 kaynaklarında şu yollar esas alındı:

- `client/multiplayer/ClientPacketListener.java:2348-2372`, mevcut entity'nin
  `LivingEntity` olması durumunda snapshot base değerini yazar, mevcut
  modifier'ları siler ve listedeki modifier'ları transient olarak ekler.
  Entity bulunamazsa listener snapshot'ı uygulatmaz.
- `world/entity/player/Player.java:206-220` ve
  `world/entity/LivingEntity.java:333-360`, Player attribute supplier'ının
  `movement_speed` ve `max_health` dahil başlangıç instance'larını kurar.
- `world/entity/ai/attributes/AttributeMap.java:45-49`, supplier'da bulunan
  attribute için instance oluşturur ve mevcut instance'ı yeniden kullanır.
- `world/entity/ai/attributes/AttributeInstance.java:139-170`, effective
  değeri `add_value` → `add_multiplied_base` → `add_multiplied_total` sırasıyla
  hesaplar ve attribute'ın sanitize yolundan geçirir.

Java oracle fixture'ı tam bir graphical `ClientPacketListener` oturumu
kurmaz; listener'daki uygulama sırasını gerçek `Player.createAttributes`,
`AttributeMap` ve `AttributeInstance` sınıfları üzerinde çalıştırır. Bu, test
izini packet dispatch ve render görevlerinden ayırırken hesaplamayı Java
class'larından alır.

## Yeni dal ve varyant kararı

Bu dilim, Java listener'ının bir attribute snapshot'ını mevcut local player
instance'ına uygulayıp tekrar gelen aynı attribute'ta önceki modifier'ları
temizlediği client-state dalını ilk kez karşılaştırır. Öncekilerden farklı
gözlem, movement speed'in üç modifier'dan `0.63` değerine, replacement
snapshot'ından `0.4` değerine geçmesi ve belirtilmeyen `max_health` state'inin
`45.0` olarak korunmasıdır. Yanlış player ID'si state'i değiştirmez. Aynı
sonucu veren mevcut `client-parity` kayıtlarının sayısı **0**'dır; P08.j ve
P08.p bu sayıya dahil değildir çünkü ikisi `server-conformance` sınıfındadır.

## Davranış sözleşmesi

- Hedef Java Edition `26.2`, protokol `776`, Java `25` ve local player ID'si
  `42`'dir. Pinned attribute registry ID'leri `movement_speed=26` ve
  `max_health=23` olarak kullanılır.
- Player ID eşleşen snapshot'ta her attribute için base değer yazılır, önceki
  modifier listesi tamamen temizlenir ve yeni modifier'lar eklenir. Snapshot'ta
  bulunmayan mevcut attribute state'i korunur.
- Player ID eşleşmeyen snapshot parse edilmiş olsa da local player world
  state'ini değiştirmez.
- Effective değer operation gruplarını Java sırasıyla işler ve sorgunun
  verdiği range içinde NaN/clamp sanitize uygular. Modifier gözlemleri
  identifier'a göre sıralanarak deterministik kaydedilir.

## Java yakalama

`tools/behavior/cases.py` içindeki
`client-world/player-attribute-application-and-effective-value` trace'i önce
movement speed ve max health snapshot'larını uygular, sonra yalnız movement
speed'i yeniler ve son olarak yanlış player ID'siyle gelen snapshot'ı yoklar.
Java oracle gerçek `AttributeMap.getInstance`, `AttributeInstance.setBaseValue`,
`removeModifiers`, `addTransientModifier` ve `getValue` çağrılarını kullanır.

Java referans kimliği:
`e75181e84b744aa3b8ccce2d5dc730c644a6fcff0b0fc9215c6037f5c66a56f8`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2968** kayıt
olarak doğrulandı ve fark `null` kaldı. İlk snapshot'ta movement speed
effective bitleri `3fe428f5c28f5c29`, replacement sonrası
`3fd999999999999a`, max health effective bitleri ise `4046800000000000` oldu.

## Rust doğrulaması

`ClientWorld::set_local_player_id` test fixture'ının local player state'ini
kurar. `ClientWorld::apply_local_player_attributes`, yalnız eşleşen ID'ye
uygulama yapar; `ClientAttributeState::effective_value` Java operation sırası
ve verilen range ile sonucu üretir. Adapter, base/modifier/effective state'i
production `ClientWorld` sorgusundan okur.

Debug adapter SHA-256 değeri
`ff3957027c6bbac24dab45cb49ee136935d22071630f18d543a0d51463489faa` ile aynı
capture üzerinde **2968/2968** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`e4bee51aeb7ce359fdaa14370261ad212e669828bf5bf23a5f026154565a32ab` ile aynı
capture üzerinde **2968/2968** eşleşti; fark `null` kaldı.

```text
cargo test --locked --offline --quiet
188 passed; 0 failed

python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/debug/examples/behavior_adapter
case_count=2968
difference=null
status=parity_passed

python3 tools/behavior/run.py compare --family math \
  --adapter /home/batuhan/git/photon/target/release/examples/behavior_adapter
case_count=2968
difference=null
status=parity_passed
```

## Açık kapsam

Bu kayıt default player attribute state'inin packet öncesi lookup'ını, tam
attribute registry isim/range çözümünü, aynı operation grubundaki Java map
sırası etkisini, remote living entity binding'ini ve movement/health/combat
gameplay etkilerini kapsamaz. Attribute'ların `LocalPlayer` tick ve hareket
formüllerine bağlanması, item/effect modifier yaşam döngüsü, ClientLevel'in
gerçek packet listener entegrasyonu ve oynanabilir client açıktır.
