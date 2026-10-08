# T37b3 — Varlık üstündeki bloğun kaplaması

**Amaç:** `BlockModelRenderState.submit`'in `overlayCoords`'u: sulfur cube'un taşıdığı blokta hasar kırmızısı ve yanık fitilin beyazı (`SulfurCubeInnerLayer`), aynı yolu kullanan öteki katmanlarda (enderman bloğu, kar golemi balkabağı) varsa kaplama. Şu an `HeldItem::block` yerdeki eşya hattından geçer; o hat (`set_ground_items`, `VERTEX_SHADER`) kaplama taşımaz.
**Bağımlılık:** T37b2. **Boy:** M.

## Oku
- Java: `client/renderer/block/BlockModelRenderState.java` (`submit`), `layers/{SulfurCubeInnerLayer,CarriedBlockLayer}.java`, `OverlayTexture.java`.
- Rust: `crates/photon/src/equipment/held_items.rs` (`Carried::Block`), `crates/photon-render/src/{item_entity,vulkan}.rs` (`entity_block_vertices`, entity ve yer hatları).

## Yapılacaklar
Önce hangi katmanların gerçekten kaplama verdiğini Java'da doğrula; kaplamayı taşıyan hatta (entity shader'ı `in_shade`'i kaplama kodu olarak okur) blok köşelerini çiz ya da yer hattına kaplama ekle. Çözüm geçici olmasın; oracle'a kaplama kodunu ekle.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: bloğun kaplama kodu (`HeldItem::block(.., overlay)`): sulfur bloğu (fitil beyazı / hasar kırmızısı), kar golemi balkabağı ve mooshroom mantarı hasar kırmızısı; öteki katmanlar `NO_OVERLAY`. Held shader çifti ışığı tint'ten ayırdı (`HELD_VERTEX_SHADER`), kaplama ışıktan önce uygulanır; kod `shade + 2·kod` ile taşınır. Yer/blok hattı entity hattından önce çizilir.
- Oracle: `cube-mob-rotations`'a 4 satır; gerçek `SulfurCubeInnerLayer`'ın `submitBlockModel` kaplama değeri. Kar golemi ve mooshroom kaplaması oracle'sız (aynı `getOverlayCoords`, bkz. Java katman kaynağı); canlıda sulfur cube + kırmızı yün gözlendi.
- Java'dan sapma: opak yer hattı ile translucent entity sırası Java'nın tek sıralı submit düzeninin yaklaşığıdır.
