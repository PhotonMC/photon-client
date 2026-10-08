# T84b — Dünya sınırının çizimi

**Amaç:** `WorldBorderRenderer`: sınırın dört yüzünün `forcefield.png` ile çizilmesi (uzaklığa göre alfa, durum rengi: durağan/küçülüyor/büyüyor, doku kayması), `BorderStatus` renkleri ve uyarı kırmızı örtüsü (`Hud`'un sınır uyarısı).
**Bağımlılık:** T84. **Boy:** M.

## Oku
- Java: `client/renderer/WorldBorderRenderer.java`, `client/renderer/rendertype/RenderTypes.java` (sınır boru hattı), `world/level/border/{WorldBorder,BorderStatus}.java`.
- Rust: `crates/photon-world/src/border.rs`, `crates/photon-render/src/{vulkan,weather}.rs` (şeffaf boru hattı örneği), `crates/photon/src/play.rs`.

## Oracle
Gerçek `WorldBorderRenderer` köşe/UV/alfa kaydı (`border_quads` operation'ı).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`; canlıda sınıra bakılır (`tools/photon.py shot`) bir kez.

## Belgeler
`docs/records/client-world-border-live-parity-conformance.md`.

## Devir notu

`crates/photon-render/src/border.rs`: `extract`/`closest_sides`/`quads`/`triangulate`, oracle'daki
`world_border_render` operation'ına (`MathShapeBorderOracle.shapeworldBorderRenderCase`,
`crates/photon-oracle/src/shape/border.rs`) karşı beş durumda bit bit eşit (`check --capture` yeşil,
yalnız bilinen Q04 farkı dışında). `BorderStatus.getColor()` artık `photon_world::ClientWorldBorderStatus::color()`;
oracle de bunu kullanıyor (kopya kaldırıldı).

Vulkan: `PipelineKind::WorldBorder` (`SkyOverlay`/`Rays` ile aynı `OVERLAY` harman dalı, kendi derinlik
kayması), `world_border.vert/.frag` (ışık haritası/sis yok — referans da sınırı ışıklandırmıyor/sislemiyor),
`misc/forcefield.png` en-yakın-komşu+döşenen örnekleyici (`load_world_border_texture`, `ticks.rs`'te
yükleniyor). `atmosphere.rs::world_border` her karede `extract`+`quads` üretip `backend.set_world_border`
ile veriyor; `sky.rs::draw` bunu hava geçişinin hemen ardından çağırıyor (`addWeatherPass`'teki sıra).
`depthFar` yarı yüksekliği `play/draw.rs`'teki `far` ile aynı formül (`render_distance*4` veya bulut
mesafesi).

Canlı doğrulama: RCON ile sınır oyuncunun yanına küçültülüp `tools/photon.py shot` ile dokulu, yarı
saydam, ton renkli duvar gözle görüldü (ekran görüntüsü Java ile piksel karşılaştırılmadı — P15.a kapsamı).

Sapma: sınır dışındayken ekran kırmızı örtüsü bu görevde yok (T83b kapsamı, kayıtta not edildi).
