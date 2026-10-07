# T40d — Düşen blok — [server-conformance]

Bu kayıt `falling_block` entity'sinin (`FallingBlockEntity`, `FallingBlockRenderer`, `MovingBlockFeatureRenderer`) çizimini kapsar. Kanıt sınıfı **`server-conformance`**tır: bu dilim için Java'ya karşı karşılaştırma yoktur; yalnız canlı sunucunun paketini kabul etmesi ve gözlem vardır (kum, çim bloğu, örs). Blok mesh'i dünya mesh'iyle aynı kodu kullanır; onun eşitliği kendi kayıtlarındadır.

## Java kaynak incelemesi

- `FallingBlockRenderer.submit`: `translate(-0,5, 0, -0,5)` ve `submitMovingBlock`; yalnız `RenderShape.MODEL` olan bloklar çizilir. `shouldRender`: yerindeki blok entity'nin durumuyla aynıysa çizilmez.
- `extractRenderState`: blok durumu `AddEntity` verisidir (`Block.getId`); çizilen blok konumu `BlockPos.containing(x, sınırKutusu.maxY, z)`, çeşit tohumu `getStartPos()` (eklendiği blok).
- `MovingBlockFeatureRenderer`: `tesselateBlock` ile tek bloğun modeli (akışkansız), `MovingBlockRenderState` etrafı hava sayar (`getBlockState` yalnız kendi konumunda dolu); ışık gerçek ışık motorundan; renk `getBlockTint`: o konumun biyomu, harmanlanmadan. Katman: katı/oyuk/saydam hareketli blok boru hatları (arazinin ışık haritası ve sisiyle).

## Yapılan

`BlockRenderer::mesh_moving_block` (`mesh_region`'un akışkansız, tohum konumlu hâli), `VulkanBackend::set_moving_blocks` (arazinin hattıyla çizilen çerçeve başına mesh'ler), oyunda `falling_blocks.rs`; entity'ye `start_block` (`getStartPos`).

## Sapmalar ve açık kalanlar

- Java karşılaştırması yok (`mesh_moving_block` ve katman ayrımı gerçek `MovingBlockFeatureRenderer`'a karşı doğrulanmadı); `cutoutLeaves` seçeneği ve zorla katı çizim (`forceOpaque`) yok.
- Düşme hareketi mevcut balistik adımlardadır; yere inen bloğun yerleşmesi sunucudan gelen blok güncellemesidir.
- Gölge (`shadowRadius 0,5`) ve sesler çizilmez.
