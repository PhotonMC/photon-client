# T80c — Bayrak bloğu

**Amaç:** `BannerRenderer` (direk, bayrak, desen katmanları, rüzgâr sallanması) ve `BlockEntityData` ile desenler.
**Bağımlılık:** T80. **Boy:** M.

## Oku
Java: `client/renderer/blockentity/BannerRenderer.java`, `state/BannerRenderState.java`, `client/renderer/special/BannerSpecialRenderer.java`. Rust: `crates/photon/src/world/block_entities.rs`.

## Oracle
Gerçek renderer kaydı `entity/chest-pose` kalıbıyla (poz, ışık, doku, dörtgenler). Java kaynağı önce satır satır okunur.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Kapsam dışı
Başka blok entity aileleri.

## Belgeler
`docs/records/client-chest-parity-conformance.md`'ye bölüm, `docs/index.md` gerekirse.

## Devir notu
Yapılan: bayrak bloğu (direk, bayrak, düz katman ve ilk on altı desen, boya, evre) gerçek `BannerRenderer`'a bit bit eşit. Kayıt: [client-chest-parity-conformance.md](../records/client-chest-parity-conformance.md).
Ek: canlı oturum `BlockEntityData` ve parça blok entity'lerini dünyaya işlemiyordu; işliyor, blok türü değişince eski blok entity'si düşüyor.
Sapma: alfa kesimi yüzde ona (Java'da yok); satır içi (veri paketi) desenler çizilmez.
Açık: sallanma canlıda hareket olarak izlenmedi.
