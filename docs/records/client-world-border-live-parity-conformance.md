# T84/T84b — Dünya sınırı canlı oturumda: paketler, çarpışma ve çizim — [client-parity] (çarpışma, çizim geometrisi) · [server-conformance] (paketler)

Dünya sınırı durumu ve şekli [önceki kayıtlarda](client-world-border-state-conformance.md) vardı; bu kayıt onu canlı oyuna bağlar.

## Java kaynak incelemesi

- `ClientPacketListener.handleInitializeBorder` ve `handleSetBorder*`: merkez, boyut (`lerp` süresi sıfırdan büyükse `lerpSizeBetween`, değilse `setSize`), en büyük boyut, uyarı süresi ve uzaklığı. Paketler: `InitializeBorder` (iki çift merkez, iki çift boyut, var long süre, üç var int), `SetBorderCenter`, `SetBorderLerpSize` (var long süre), `SetBorderSize`, `SetBorderWarningDelay`, `SetBorderWarningDistance`.
- `Entity.collectColliders` (`collectAllColliders`): kaynak varlık sınırın yakınındaysa (`WorldBorder.isInsideCloseToBorder`) sınırın çarpışma şekli blok çarpışmalarına eklenir; `CollisionGetter.noCollision(entity, kutu)` da sınırı sayar.
- `WorldBorderRenderer.extract`: kamera sınırın `renderDistance` içindeki bir kenarına yakınsa ve genişletilmiş kutunun içindeyse, alfa `(1 - en yakın kenara uzaklık / renderDistance)^4` (0–1 kırpılmış), ton `BorderStatus.getColor()` (GROWING/SHRINKING/STATIONARY). `rebuildWorldBorderBuffer` dört yüzü (güney/batı/kuzey/doğu, `closestBorder`'ın verdiği sırayla) `Mth.floor/ceil` ile kamera merkezli döşenmiş UV'lerle kurar; `render` yalnız `renderDistance` içindeki yüzleri çizer, dokuyu `Util.getMillis() % 3000` ile kaydırır ve `BlendFunction.OVERLAY` (`SRC_ALPHA,ONE` renk; `ONE,ZERO` alfa) ile harmanlar.

## Yapılan

- Altı paketin ayrıştırması (`protocol_tests`) ve `ClientWorld`'e uygulanması; sınır her istemci tikinde zaten ilerliyordu.
- `ClientWorld::border_collider(kaynak, kutu)`; `Level::{colliders, no_collision}` kaynağın konumunu alır, sınır şeklini başa ekler. Yerel oyuncu, binekler ve yerdeki eşya bununla çarpışır; otomatik zıplamanın kutu taraması yalnız bloklara bakar (Java'daki gibi).
- Kanıt (`client-parity`): `movement/local-player-ticks` sahnelerine `border` alanı (merkez, boyut) ve beş satır: sınıra yürüme, koşarak zıplama, köşede kayma, dışarıdan başlama, eğilerek yürüme; gerçek `LocalPlayer` ile bit bit eşit.
- `crates/photon-render/src/border.rs`: `extract` (alfa, ton), `closest_sides` (`closestBorder`'ın dört yönü, uzaklığa göre kararlı sıralı) ve `quads` (`rebuildWorldBorderBuffer`/`render`'ın köşe, UV ve kayma matematiği), dörtgenleri üçgene bölen `triangulate`. Kanıt (`client-parity`, `shape/world-border-render` operation'ı, `MathShapeBorderOracle.shapeworldBorderRenderCase`): gerçek `WorldBorderRenderer.extract`/`WorldBorderRenderState.closestBorder` ile (Unsafe ile GPU tamponsuz ayrılmış), durağan/büyüyen/küçülen sınır, köşede eşit uzaklıklı iki yüz (kararlı sıralama), sınır dışı (sıfır alfa) ve farklı `half_height_y`/`time_millis` kapsayan beş durumda alfa, ton ve her köşenin x/y/z/u/v'si bit bit eşit.
- Vulkan: yeni `PipelineKind::WorldBorder` (`world_border.vert`/`.frag`, `BlendFunction.OVERLAY`'in `SkyOverlay`/`Rays` ile paylaştığı harman, `DepthStencilState`'in 3,0/3,0 derinlik kayması), `misc/forcefield.png` dokusu (en yakın komşu, döşenen örnekleyici), `crates/photon/src/world/atmosphere.rs::world_border` her karede `extract`+`quads` çağırıp `backend.set_world_border` ile veriyor (`LevelRenderer.addWeatherPass`'teki gibi hava efektiyle aynı geçiş). `depthFar` `Camera.update`'teki gibi `max(görüş mesafesi*4, bulut mesafesi*16)`.
- Canlıda bakıldı (`tools/photon.py shot`): sınır merkezi ve boyutu RCON ile oyuncunun yanına küçültülüp dokulu, yarı saydam, ton renkli duvar gözlemlendi.

## Sapmalar ve açık kalanlar

- Sınır dışındayken hasar kırmızısı (ekran örtüsü) ayrı kapsam: [T83b](../tasks/T83b-ekran-orturleri.md).
- `getDamagePerBlock`, `safeZone` paketlerle gelmez (sunucu hasar verir); istemcide okunmaz.
- Sınırı bekleyen sunucu-uyum kaydı yok. Canlı çizim yalnız gözle doğrulandı; Java'nın kendi ekran görüntüsüyle piksel piksel karşılaştırma yok (P15.a kapsamı).
