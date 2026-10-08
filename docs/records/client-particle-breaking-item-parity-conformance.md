# T77b — Kırılan eşya parçacıkları — [client-parity]

Bu kayıt `BreakingItemParticle`'ı ve sağlayıcılarını kapsar: `item`, `item_slime`, `item_cobweb`,
`item_snowball`, `sulfur_cube_goo`. Ayrıca `ItemParticleOption`'ın ağdan okunmasını içerir. Kanıt
sınıfı `client-parity`'dir. `movement/particle-ticks`'te goo gerçek sağlayıcıyla koşar. Eşya
kırıntısı başsız ortamda eşya modeli çözücüsü olmadığından Java'da kurucusuyla (sekiz ve beş
argümanlı) kurulur. Hepsi bit bit karşılaştırılır.

## Java kaynak incelemesi

- Kırıntı yerçekimi 1 ile düşer, boyu yarıdır. Resminin dörtte biri, `uo, vo = r*3` yerinden
  gösterilir (`getU0 = (uo+1)/4`). Katmanı resmin saydamlığından seçilir (Rust kesik çizer).
- Sekiz argümanlı kurucu kendi yayılımının onda birini tutar ve verilen itişi ekler. Slime topu,
  örümcek ağı ve kartopu sağlayıcıları itişsiz kurucuyu kullanır.
- Resim eşyanın yerdeki modelinin katmanlarından birinin parçacık dokusudur
  (`ItemStackRenderState.pickParticleMaterial`: sağlayıcı rastgelesinden `nextInt(katman sayısı)`).
  Katman yoksa eksik resim kullanılır. Goo kendi parçacık resminin ilkini alır.
- `ItemParticleOption` ağda eşya, adet ve bileşen yaması olarak gelir.

## Kapsam ve dal kararı

Üç satır var: itişli ve itişsiz kırıntı, goo. Rust'ta eşya kırıntısı blok atlasında (`Terrain`
penceresi), goo parçacık atlasında pencereli çizilir.

## Sapmalar (açık)

- Eşya modelinden resim seçimi oracle'da sınanmıyor; bileşen yaması modeli değiştirmiyor (okunup
  atlanıyor).
