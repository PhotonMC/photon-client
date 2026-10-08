# T52 — Bileşenli eşyada tıklama paketi hash'i ve bundle tıklaması

**Amaç:** Tıklama paketi bileşeni olan yığını bileşensiz hash'liyor; `HashedStack` bileşen hash'i (CRC32C), bundle tıklama geçersiz kılmaları (`SelectBundleItem`, bundle üstüne tıklama), zırh kilidi büyüsü ve hasarlı eşya takası tahmini.
**Bağımlılık:** –. **Boy:** M.

## Oku
Java: `network/HashedStack.java`, `network/HashedPatchMap.java`, `world/inventory/AbstractContainerMenu.java` (`doClick`), `world/item/BundleItem.java`, `world/item/component/BundleContents.java`. Rust: `crates/photon-game/src/menus/container.rs`, `crates/photon-net/src/connection/protocol.rs` (tıklama kodlayıcıları).

## Oracle
`movement/container-clicks` satırları: bileşenli eşya (özel ad, büyü), bundle alma/koyma, `SelectBundleItem` baytı; paket baytı Java ile birebir. G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/photon.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/records/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
Yapılan: `HashOps` (CRC32C) ve 14 sık bileşenin hash'i (`photon-world/src/item_components/hash.rs`),
tıklama paketinde eklenen/kaldırılan bileşenler (`stacks.rs` `write_hashed_stack`), büyü adları
oyunda kayıt senkronundan (`containers.rs`). Oracle: `movement/component-hashes` (16 satır) ve
`movement/container-clicks`'e 4 bileşenli satır; Java tarafında gerçek hasher.
Sapma: çok bileşenli yığında bileşen sırası Java'da kimlik hash'ine bağlı (rastgele), Rust yama sırası.
Açık: kalan bileşen türlerinin hash'i yazılmıyor; bundle, hasarlı eşya, zırh kilidi → T52b.
Kapı: yalnız bilinen Q04 `particle-engine` farkı kırmızı.
