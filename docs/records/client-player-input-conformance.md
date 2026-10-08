# P10.a — LocalPlayer input modifiers — [client-parity] birleşik kayıt

Bu kayıt, Java `LocalPlayer.modifyInput` dönüşümünü `KeyboardInput` snapshot'ı
sonrasındaki bağımsız input helper'ı olarak karşılaştırır. Kanıt sınıfı
`client-parity`'dir: aynı sentetik `Vec2` ve modifier girdileri Java 26.2
oracle'ında ve Rust production helper'ında çalıştırılır. Tam hareket tick'i,
çarpışma ve oyuncu world state'i bu kaydın kapsamına girmez.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 incelemesinde şu yollar esas alındı:

- `client/player/LocalPlayer.java:692-736`, input'un sıfır erken dönüşünü,
  `0.98F` taban çarpanını, eşya kullanımı ve yavaş hareket çarpanlarını ve
  kare hareket düzeltmesine geçişi tanımlar.
- `world/phys/Vec2.java:29-30,59-70`, input uzunluğu, normalize yönü ve f32
  karekök yolunu tanımlar.
- `util/Mth.java:58-60`, karekökün Java f32 dönüş davranışını sağlar.
- `world/item/component/UseEffects.java:9-10`, eşya kullanım hız çarpanının
  kaynağını taşır.
- `world/entity/ai/attributes/Attributes.java:92`, varsayılan sneaking speed
  attribute değerini sağlar.

## Davranış sözleşmesi

- `input.lengthSquared() == 0.0F` ise input aynen döner; bu erken dönüş signed
  zero bileşenlerini de korur.
- Sıfır olmayan input önce `0.98F` ile ölçeklenir.
- Oyuncu eşya kullanıyor ve passenger değilse `UseEffects.speedMultiplier`
  uygulanır. Passenger durumunda bu çarpan atlanır.
- Oyuncu yavaş hareket ediyorsa sneaking movement attribute değeri uygulanır;
  çarpanların sırası değiştirilmez.
- Kare hareket düzeltmesi input yönünü normalize eder, büyük mutlak eksene göre
  tangent seçer, `sqrt(1 + tangent²)` mesafesini hesaplar ve uzunluğu birim
  kareye göre `min(..., 1.0F)` ile sınırlar.
- Karşılaştırma f32 ham bitleriyle yapılır. Java işlem sırası, f32 yuvarlama,
  signed zero ve özel değer davranışı için epsilon kullanılmaz.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik `player-input/modify` kaydı
on gözlem taşır:

| Gözlem | İlk kez sınanan dal |
| --- | --- |
| `zero-preserves-signed-zero` | sıfır input erken dönüşü |
| `axis-applies-base-factor` | `0.98F` taban çarpanı |
| `diagonal-clamps-to-unit-square` | diyagonal kare düzeltmesi ve clamp |
| `short-vector-keeps-scaled-length` | birimden kısa vektörde yön/uzunluk |
| `item-use-multiplier` | passenger olmayan eşya kullanımı |
| `item-use-bypassed-in-passenger` | passenger eşya kullanım bypass'ı |
| `sneaking-attribute-factor` | yavaş hareket attribute çarpanı |
| `item-and-sneaking-order` | eşya ve sneaking çarpanlarının sırası |
| `zero-item-factor-reaches-square-zero` | çarpan sonrası kare düzeltmesi sıfır yolu |
| `non-unit-vector-clamps` | birim dışı input'un uzunluk clamp'i |

Java oracle, pinned JAR içindeki gerçek private `LocalPlayer.modifyInput`
metodunu kontrollü fixture ve reflection ile çağırır. Beklenen sonuç Rust
çıktısından türetilmez.

Java referans kimliği:
`e85c7af04e355556103bbe191711d539f20c2a7da92b6fd68d821150a4f3cb0a`.

Capture iki Java çalıştırmasında kararlı kaldı; math corpus'u **2950** kayıt
olarak doğrulandı ve fark `null` kaldı.

## Rust doğrulaması

`crates/photon-world/src/input.rs` içindeki `modify_local_player_input`, LocalPlayer dönüşümünü
production kodunda uygular. Behavior adapter aynı helper'ı çağırır; adapter
ayrı bir matematik kopyası taşımaz.

Debug adapter SHA-256 değeri
`69fdeb6d5deade5ef33efbcab78901c1cf505eda3dd8b09bf8fe7b6842230fa2` ile
**2950/2950** eşleşti; fark `null` kaldı.

Release adapter SHA-256 değeri
`47c8d2afbde3857407339ea130a76cadd8f305c7162d516c4e9dcada70f43f16` ile aynı
capture üzerinde **2950/2950** eşleşti; fark `null` kaldı.

Dilimde kullanılan temel kontroller:

```text
python3 tools/behavior/run.py capture --family math --jdk-home /usr/lib/jvm/java-25-openjdk
python3 tools/behavior/run.py verify --family math --jdk-home /usr/lib/jvm/java-25-openjdk
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /usr/lib/jvm/java-25-openjdk --adapter target/release/examples/behavior_adapter
```

## Vanilla server-conformance adjunctı

Mojang vanilla 26.2 dedicated server JAR'ı
`823e2250d24b3ddac457a60c92a6a941943fcd6a` SHA-1'iyle doğrulandı. Sunucu
offline ve yalnızca `127.0.0.1:25568` üzerinde, seed `776`, creative,
peaceful, view/simulation distance `5` ayarlarıyla geçici
`/tmp/photon-vanilla-26.2-test` dizininde çalıştırıldı. JAR, dünya ve loglar
depoya alınmadı.

Mevcut movement wire probe şu komutla resmi vanilla sunucusuna karşı
çalıştırıldı:

```text
cargo run --locked --offline --example movement_probe -- 127.0.0.1:25568 P10aVanilla
```

`server-conformance` koşusu login ve `player_loaded` geçişlerini tamamladı;
üç movement paketi kabul edildi, 24 chunk paketi çözüldü ve oturumda 312
paket alındı. Bu probe mevcut hardcoded movement wire yolunu çalıştırır;
`modify_local_player_input` henüz gerçek ClientWorld/local player movement
tick'ine bağlanmadığı için bu sonuç helper'ın oynanabilir istemciye bağlı
olduğunu veya Java istemcisiyle parity sağladığını göstermez.

## Açık kapsam

Input helper'ının LocalPlayer tick'ine bağlanması, konum/hız entegrasyonu,
collision çözümü, pose ve sprint geçişleri, jump/crouch, su/lav/merdiven,
özel yüzeyler, uçuş/elytra/auto-jump, araç kontrolü ve serverbound movement
senkronizasyonu P10.b–e ve P11 kapsamındadır.
