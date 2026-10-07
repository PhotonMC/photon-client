# T70c — Partikül dörtgenleri ve çizimi — [client-parity] ve [server-conformance]

Bu kayıt, bir partikülün çizim için yazdığı dörtgenin (`SingleQuadParticle.extract` ve
`QuadParticleRenderState.renderRotatedQuad`) Java 26.2 ile Rust'ta aynı olmasını kapsar.
`client-parity`: gerçek partikül tik tik ilerletilir, gerçek `Camera` (konum, `setRotation`) için
`extract` çağrılır ve katmanın yazdığı dört köşe (konum, doku koordinatı, `ARGB` renk, paketli ışık)
bit bit karşılaştırılır. `server-conformance`: canlı sunucunun `/particle` komutuyla gönderdiği
alev, küçük alev ve duman istemcide çizilir (`tools/voxel.py shot`, yalnız görsel gözlem; sunucu
paketini kabul etmesi eşitlik kanıtı değildir).

## Java kaynak incelemesi

- `SingleQuadParticle.extract`: dönüş kameranın dönüşüdür (`LOOKAT_XYZ`), `roll` sıfır değilse son
  ve yeni değer arasında (`lerp`) z ekseninde ayrıca döndürülür; konum `(float)(lerp(partial, xo, x) - kamera)`;
  boyut `getQuadSize(partial)`; renk `ARGB.colorFromFloat` (her kanal `floor(c * 255)`).
- `QuadParticleRenderState.renderRotatedQuad`: dört köşe `(1,-1)`, `(1,1)`, `(-1,1)`, `(-1,-1)`; her biri
  dönüşle döndürülür (`Vector3f.rotate(Quaternionf)`, `fma`lı), ölçeklenir, konuma eklenir; ışık
  `setUv2(light & 65535, light >> 16)`.
- `QuadParticleGroup`: görüş piramidinde olmayan partikül (`pointInFrustum`) yazılmaz.
- Çizim katmanı `OPAQUE` (parçacık atlası, ana alfa kesmeli boru hattı): duman ve alev bu katmandadır.

## Kapsam ve dal kararı

`movement/particle-quads` 10 satır: dört bakış yönü ve kısmi tik (kameranın dönüşü), uzak köken
(kayan nokta duyarlığı), duman, alev ve küçük alev, ve iki `roll` satırı (z dönüşü). Rust çizimi
bunun üzerine kuruludur: partikül atlası ayrı yapılmaz, partikül dokuları (`textures/particle/*`)
varlık doku atlasına paketlenir ve dörtgenler varlık boru hattıyla (`Run::Particle`,
derinlik yazan, alfa kesmeli, ışık haritalı) yeni `Pass::Particles` geçişinde, yarı saydam arazi ile
bulutlar arasında çizilir.

## Sapmalar (açık)

- Yarı saydam katman (`TRANSLUCENT`) ve kendi atlası: ilk yarı saydam partikül sınıfı gelince.
- Partikül atlası Java'daki gibi mipmap'li ayrı atlas değil; varlık atlasının örneklemesi geçerli.
- `Frustum.pointInFrustum` yerine sıfır boyutlu kutunun görünürlüğü kullanılır.
- Sprite adı `minecraft:` ad alanı dışına çıkmaz (`particle/<ad>`).
