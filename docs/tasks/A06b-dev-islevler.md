# A06b — Tek dev işlev olan dosyalar ve oracle adaptörünün crate'i

**Amaç:** A06'nın bölünemeyen kalanı: 600 satırı aşan işlevleri/dosyaları işlev başına bölmek.
**Bağımlılık:** A06. **Boy:** L.

## Oku
- [A06 devir notu](A06-dev-dosyalar.md) (kalan dosyaların listesi), [dosya boyutu kuralı](../coding-standards.md#dosya-boyutu)

## Yapılacaklar
- Adaptörde `match operation`/`if operation == …` kolları işlev başına (`*_result`) çıkarılır, dağıtım bir `(ad, işlev)` tablosuna iner; `math_cases.rs`, `shape.rs`, `ticks_2.rs`, `gui/result.rs`, `entity.rs`.
- `probe/mode.rs`: test izleme (A04 `Recorder`) ile birlikte aşamalara; `play/incoming`: paket başına işleyici (A04).
- `outline.rs`: tek işlev, bloğa göre alt işlevler.
- Python: `math_cases` üreticisi konuya göre alt üreticilere; `cases.py` bir pakete (`cases/__init__.py` + konu modülleri), vaka listesinin bayt eşitliği önce/sonra hash'iyle doğrulanır.
- Java oracle sınıfları statik yöntem gruplarına göre sınıf zincirine (veya ayrı sınıflara) bölünür; runner derlemesi ve `check --capture` ile doğrulanır.
- `crates/voxel-oracle` (publish=false) crate'i: adaptör `examples/`'tan taşınır.

## Kabul
Hiçbir kod dosyası 600 satırı aşmaz; `python3 tools/voxel.py check --capture` → `CHECK OK`; vaka listesi ve Java çıktıları önceki capture ile eşit.

## Kapsam dışı
Davranış değişikliği; üretilmiş tablolar (D görevleri).

## Belgeler
Bu görev dosyasının devir notu.

## Devir notu
- **Adaptör bitti:** `math_cases.rs` (254 `if operation == …` bloğu → konuya göre `math_cases/{positions/{blockpos,chunkpos,sectionpos},direction,vectors,mth/{interpolation,angles,numbers,algorithms},player,options}.rs`, dağıtım bir `match`), `shape.rs` (68 kol → `shape/{packets,boxes,rotation,collision,world_collision,world_index,border,dimensions,queries}.rs`), `entity.rs` (`entity/{tables,player}.rs`), `gui/result.rs` (`GuiContext` + senaryo başına işlev, `gui/result/{chat,hud,screens,settings,container}.rs`), `living_ticks.rs` (kurulum + `events/{world,mobs,objects}.rs` + `extras/{animals,aquatic,special,objects}.rs`). Bloklar sözcüğü sözcüğüne işlevlere taşındı (son `return X;` → `X`); 542 test ve 1109 parity kaydı aynı.
- `play/incoming.rs` `handle` paket kolları `on_<paket>` yöntemlerine (`incoming/{entity,world,player,containers,chat,recipes,connection}.rs`), `handle` yalnız dağıtım; `block/shape_rules/singles/outline.rs` sınıf ailesine göre (`outline/{mounts,plants,redstone,furniture}.rs`). Davranış aynı.
- **Kalan:** `probe/mode.rs` (2267; tek `while` döngüsü ve ~30 yerel durum değişkeni, `Recorder` ile birlikte A04'te); Python `cases.py` (19.4k), `cases_gui.py` (2.3k), `tools/voxel.py` (740); Java oracle'lar (`MathOracle` 13k, `EntityOracle` 6.2k, `MovementOracle` 3.1k, `GuiOracle` 2.8k, `BlockOracle` 1.7k, `WireOracle` 1.5k, `SoundOracle`, `ModelOracle`); `crates/voxel-oracle` taşıması.
