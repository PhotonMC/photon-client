# T55 — Tüccar menüsü ve `MerchantOffers`

**Amaç:** `MerchantMenu` (iki ödeme yuvası, sonuç yuvası, `MerchantResultSlot`), `ClientboundMerchantOffersPacket` ayrıştırması (`MerchantOffers`: maliyet A/B, sonuç, kullanım, azami, indirim), `SelectTrade`, `MerchantScreen` (teklif listesi, kaydırma, ilerleme çubuğu, ipucu), `T34`'ün köylüsüne bağlama.
**Bağımlılık:** T50. **Boy:** L.

## Oku
Java: `client/gui/screens/inventory/MerchantScreen.java`, `world/inventory/{MerchantMenu,MerchantContainer,MerchantResultSlot}.java`, `world/item/trading/{MerchantOffers,MerchantOffer,ItemCost}.java`, `network/protocol/game/{ClientboundMerchantOffersPacket,ServerboundSelectTradePacket}.java`. Rust: `crates/voxel-net/src/connection/{protocol,play}.rs`, `crates/voxel-game/src/container_menu.rs`.

## Oracle
Paket: `wire` ailesi `merchant_offers` çözümü; menü: `container-clicks`; ekran: `container_screen` merchant (liste kaydırma, seçili teklif, kullanılmış teklif çarpısı, ilerleme). G tarifi ve `movement/container-clicks` (README, `docs/gui-oracle.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız ekran açılıyorsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka menü türleri (kendi görevlerinde); tarif kitabı kaplaması.

## Belgeler
`docs/generic-container-conformance.md` (ilgili bölüm), `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3 işareti.

## Devir notu
(boş)
