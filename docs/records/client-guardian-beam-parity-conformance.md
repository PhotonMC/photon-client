# T35i — Gardiyanın ışını — [client-parity]

Bu kayıt, gardiyanın ışınının geometrisinin (köşelerin yeri, doku koordinatı, renk, aydınlık, normal) Java
26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı `client-parity`'dir: köşeler gerçek
`GuardianRenderer.renderBeam`'in `submitCustomGeometry` ile verdiği köşe tüketicisinden yakalanır. Canlı
gözlem `server-conformance`.

## Java kaynak incelemesi

- `renderBeam`: ışının uzunluğu `|vektör| + 1`; yönü iki döndürmeyle (`YP` ve `XP`, `acos(y)` ve
  `π/2 − atan2(z, x)`) eksene oturtulur. Dönme açısı `süre × 0,05 × −1,5`; yan yüzler 0,2 yarıçaplı dört
  köşeden (iki karşılıklı çift), uç yüzü 0,282 yarıçaplı dört köşeden kurulur. Renk `64 + 191 k²`,
  `32 + 191 k²`, `128 − 64 k²` (k = saldırı ölçeği); aydınlık tam (`0xF000F0`); normal `(0, 1, 0)`'ın duruşla
  döndürülmüşü. Yan yüzün `v` aralığı `−1 + kayma` ile uzunluk × 2,5 kadar ilerler; uç yüzün dokusu çift ve
  tek tikte yarım doku kayar.
- Işın, varlığın gövde dönüşünden bağımsızdır: ayaklardan `eyeHeight` kadar yukarıdan çıkar, hedef gözden
  hedefin boyunun yarısına uzanır; `texVOff = saldırı süresi × 0,5 mod 1`.
- Referansın dokusu tekrarlanır (`v` birden çok kez sarar); paketlenmiş görüntü tekrarlanmaz. Bu yüzden
  yan yüz her tam `v` sınırında kesilir ve parçaların `v`'si kendi tekrarından başlar; çizilen resim aynıdır.
  Kesme oracle ile karşılaştırılmaz, bir birim testle denetlenir.
- Hedef, `Guardian` verisinde kimliği olan varlıktır: yerel oyuncu ya da dünyadaki varlık (bulunamazsa ışın
  çizilmez; `getActiveAttackTarget` boş döner).

## Kapsam

`entity/guardian-beam` 14 satır (her eksen boyunca ve eğik, kısa ve uzun, hedefe oturmuş ve sıfır uzunluk,
saldırının başı, tek tik, dolu, elder süresi, kayık göz, uzak). Çizim (`crates/voxel/src/guardians.rs`) canlı
gözlenmedi.

## Sapmalar (açık)

- `Math.acos` ve `Math.atan2` Rust'ın `f64` işlevleriyle alınır; son bit farkı çıkarsa oracle yakalar (14 satırda
  çıkmadı).
- Yerel oyuncunun boyu sınırlayıcı kutudan okunur. Görüş alanı eleme (`shouldRender`) yok.
- Başın hedefe dönmesi, kabarcık parçacıkları ve karadaki çırpınış T35j'de.
