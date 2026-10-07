# P17.a — Hareket fixture'ı — altyapı kaydı

**Tarih:** 2026-09-27 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25

Bu bir parity kaydı **değil**. Bir engelin kaldırıldığını ve nasıl
kaldırıldığını yazıyor.

## Engel

Hareketin yaprakları pinliydi — `travel` sevki ve beş kolu, sürtünme, koşma
kapısı, sinsi çarpanlar, çarpışma çözüm ilkeli, basamak çıkma. Ama **bileşim**
pinli değildi: `Entity.move` ve `aiStep` hiç sürülmemişti ve her kayıt
"oynanabilir istemci açıktır" diye bitiyordu.

Sebep şuydu: bu metotlar bir `Level` istiyor, `Level` ise oracle'da
kurulamıyor sayılıyordu. Yapraklar bu yüzden `Unsafe.allocateInstance` ile
kurulmuş boş bir nesne üzerinde, çözülmüş girdilerle sürülüyordu.

Yaprakları pinlemek yetmez. `aiStep` içindeki **sıra** ve adımlar arasındaki
yuvarlama sonucu değiştirir; "her parça ayrı ayrı doğru" ile "zincir doğru"
aynı şey değildir.

## Ölçüm

`Level` kurulabiliyor. Soyut metot sayısı yirmi ve çoğu hareket yolunun hiç
uğramadığı şeyler (ses, patlama, skorbord, harita). Kalanı için gereken tek
şey kayıt defteriydi, o da [P16.s](block-registry-conformance.md) ile çözüldü.

`MovementOracle.FixtureLevel` blokları düz bir haritada tutuyor ve çarpışma
yürüyüşü bir yığın istediğinde kendini veriyor. Yürüyüş, şekil sorgusu,
çözüm ve `move`'un tamamı referansın kendi kodu; fixture'ın verdiği tek şey
bunların içinde karar verileceği bir yer.

Doğrulandı: bir zırh standı `y=70`'ten düşüp `y=64.0`'te `onGround=true` ile
duruyor; duvara yürüyünce `x=1.75`'te `horizontalCollision=true` ile duruyor.

## Üç senaryo adı yanlış iddia ediyordu

İlk yakalamada üç satır adının söylediğini sürmüyordu ve düzeltildi. Bu,
"varyant değil dal" kuralının ta kendisi: kapsadığını sandığın ama kapsamayan
bir senaryo, hiç senaryo olmamasından kötüdür.

| Satır | Sorun | Düzeltme |
| --- | --- | --- |
| `walk-into-corner` | İki direk arasından sıyrılıyordu | Gerçek iç köşe: iki duvar |
| `walk-on-honey` | Her tick taze delta veriliyordu, hız çarpanı atılıyordu | Hareket tick'ten tick'e taşınıyor |
| `bounce-on-slime` | Hiç sıçramıyordu | Aşağıdaki bulgu |

## Bulgu: istemci tarafında yalnızca oyuncu tam yolu koşar

`Entity.move` içinde sıçrama (`restituteMovementAfterCollisions`) ve düşme
hasarı `canSimulateMovement()` arkasında; o da `isLocalInstanceAuthoritative()`.
İstemci tarafı bir seviyede bu yalnızca **yerel istemcinin yetkili olduğu**
varlık için doğru, yani oyuncu için. Başka bir varlık o adımları atlar.

İki satırla pinlendi: `land-on-slime` sunucu tarafında `-0.88`'den `+1.0047`'ye
sıçrıyor, `land-on-slime-clientside` aynı sahnede hiç sıçramıyor.

Bal karşılaştırması da netleşti: `walk-on-honey` hareketi `0.83`'e
yakınsatıyor, `walk-on-stone-carried` ise tick başına `0.2` ile gidiyor.

## Fixture'ın sınırı

Bloklar yığınlarda değil düz tutuluyor, bu yüzden yığın soran her şey "yok"
cevabı alıyor. Önemli olan tek çağıran sıvı etkileşim denetimi ve o bunu
"sıvı yok" diye okuyor — bu senaryolar için doğru, ama **su ve lav senaryoları
bu fixture'la bu hâliyle sürülemez**.

## Neden korpusa eklenmedi

`movement` ailesi ve `block/face-occlusion` senaryosu kayıtlı değil: Rust
tarafları henüz yok, kayıtlı olsalar `compare` kırılırdı. Oracle işi commit
edildi, korpus karşılaştırma gerçekten koşabildiğinde büyüyecek.

Rust tarafının önkoşulu blok çarpışma şekli tablosu. Şu anki
`ClientBlockStateRegistry` kasıtlı olarak küçük bir alt küme taşıyor; kayıt
defteri ve yüz örtülmesi için işe yarayan "tabloyu referanstan veri olarak al"
yöntemiyle tamamlanmalı.

## Sıra

1. Blok çarpışma şekli tablosu (kayıt defteri ölçeğinde, pinli)
2. `Entity.move` bileşiminin Rust'ta kurulması ve bu aileyle karşılaştırılması
3. `LivingEntity.aiStep` + `LocalPlayer.aiStep` — girdi, zıplama, koşma
4. Zincirin `tick_local_player`'a bağlanması
5. Pencereden girdi ve kameranın oyuncu dönüşünden gelmesi
6. Serverbound `sendPosition`
