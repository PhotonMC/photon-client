# T39f — Olta şamandırası — [client-parity]

Bu kayıt `fishing_bobber`'ın çizimini kapsar: şamandıra sprite'ı (`FishingHookRenderer.submit`'in ilk özel geometrisi) ve oltacının eline uzanan çizgi. Kanıt sınıfı `client-parity`'dir: sprite, gerçek `submit`'in vertex tüketicisine verdiğine; çizgi, gerçek `stringVertex`'in on altı adımlık çıktısına karşı karşılaştırılır. Elin konumu oracle ile **karşılaştırılmaz** (aşağıda).

## Java kaynak incelemesi

- Sprite: 0,5 ölçek, `camera.orientation` ile kameraya dönük, `x∈[-0,5; 0,5]`, `y∈[-0,5; 0,5]`, `entityCutoutCull`; tek normal `(0, 1, 0)`'ın pose'la dönmüşüdür. Dörtgen kameraya baktığı için sarım ön yüzdür; ayıklama bir şeyi elemez.
- Çizgi: `RenderTypes.lines()`, siyah, genişlik `appropriateLineWidth`; 16 adım, her biri iki uç (`a0→a1`, `a1→a0`); `y = ya·(a² + a)/2 + 0,25` ile sarkar. `lineOriginOffset` = el − (şamandıra + `0,25`).
- Sahip: `ClientboundAddEntityPacket.getData` oltacının kimliğidir; oyuncu değilse şamandıra çizilmez. Elin konumu: üçüncü şahıs/başkası için gövde dönüşünün düz aradeğeri, `0,35·ölçek` yana, `0,8·ölçek` öne, `−0,45·ölçek` aşağı (eğilmişse `−0,1875`); kendi kamerasından birinci şahıs için yakın düzlemde `(±0,525, −0,1)` noktası, `960/fov` ile büyütülür ve salınımla (`sin(√saldırı·π)`) döner. Olta ana elde ise ana kol, değilse karşı kol.

## Kapsam

- `entity/fishing-hook` 8 satır (kamera dört yön ve eğik, el altta/üstte/uzakta, uzak konum, sıfır uzunluk).
- Rust: `projectile::fishing_hook_quad`, `photon-render::fishing` (çizgi, el, yakın düzlem), oyunda `crates/photon/src/objects/fishing_hooks.rs`; çizgi blok anahattıyla aynı çizgi hattından geçer.

## Sapmalar ve açık kalanlar

- **El konumu oracle'sız**: Java'da `getPlayerHandPos` gerçek bir `Player` ister; Unsafe ile kurulamadı. Formül kaynaktan satır satır yazıldı, yalnız iki özellik birim testindedir. Birinci şahısta kamera vektörleri JOML'un dörtlü dönüşümü yerine matrisle döndürülür (son basamak farkı olabilir).
- Yerel oyuncunun ana kolu sağ varsayılır (bağlantı bilgisi sabit), ölçeği 1'dir.
- Şamandıranın istemci tiki (uçuş, yüzme, ısırma sarsıntısı) yok: konum sunucunun gönderdiğidir ([T39g](../tasks/T39g-mermi-hareketi.md) kapsamına alınabilir). Sesler ve partiküller yok.
- Canlı: kendi şamandıram birinci şahıs görünümünde suda, çizgi çubuğun ucundan sarkarak görüldü; üçüncü şahıs ve başkasının şamandırası canlıda görülmedi.
