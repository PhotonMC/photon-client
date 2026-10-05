# T37d — Warden

**Amaç:** `WardenModel` (`KeyframeAnimation`: yürüyüş, saldırı, kükreme, ses dalgası, emerge/dig durumları), `WardenRenderer` ve katmanları (kalp atışı, parlayan göğüs ve dokunaçlar, emissive katman), `Warden` istemci animasyon durumları.
**Bağımlılık:** T37, T43b. **Boy:** L.

## Oku
- Java: `world/entity/monster/warden/Warden.java`, `client/model/monster/warden/WardenModel.java`, `client/renderer/entity/{WardenRenderer,state/WardenRenderState}.java`, `layers/LivingEntityEmissiveLayer.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`animate`, `setup_*`), `crates/voxel-world/src/animation_state.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `setup_warden` (bakış, yürüyüş, boşta, dokunaç ve altı animasyon; beş katmanın hepsi), `voxel-world/warden.rs` (poz, olay 4/61/62, `CLIENT_ANGER_LEVEL`, kalp ve dokunaç nabzı, eklenme verisi 1 = çıkış), `monsters.rs`'e warden ve dört parlayan katman, `mob_vertices_emissive_faded`; entity gölgelendiricisine köşe alfası. Kayıt: `docs/client-warden-parity-conformance.md`.
- Karar: katman alfası yeni boru hattı yerine negatif ışığın ikinci değerinde taşınır (breeze'deki bant içi işaret kalıbı).
- Sapma: parlayan katmanlar derinliğe yazar, `order(1)` sıralaması yok; kalp sesi, kazma parçacıkları ve dokunaç tıkırtısı yok. Canlı gözlem yapılmadı.
- Açık: `WardenRenderer`'ın gölge yarıçapı ve ad etiketi genel yolda; ayrıca denetlenmedi.
