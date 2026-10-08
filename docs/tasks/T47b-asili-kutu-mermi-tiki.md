# T47b — Asılı entity kutusu ve kalan mermilerin istemci tiki

**Amaç:** T47'nin oracle'ında çıkan iki boşluğu kapatmak. (1) Eşya çerçevesi, parlayan çerçeve ve
tablonun kutusu Java'da asıldığı yöne göre kurulur (`HangingEntity.calculateBoundingBox`,
`ItemFrame.createBoundingBox`: duvardan 0.46875 içeride, yön ekseninde 0.0625 kalın, haritalıysa
1×1); Rust düz `dimensions` kutusunu kullanıyor (hedefleme ve ayıklama yanlış). (2) İstemci yalnız
`arrow` ve `snowball` mermilerini kendi tikinde ilerletiyor; trident, spectral ok, yumurta, inci,
iksirler ve deneyim şişesi Java'da düşerken burada duruyor.
**Bağımlılık:** T47. **Boy:** M. **Türler:** item_frame, glow_item_frame, painting, trident,
spectral_arrow, egg, ender_pearl, splash/lingering_potion, experience_bottle.

## Oku
- Java: `world/entity/decoration/{HangingEntity,BlockAttachedEntity,ItemFrame,Painting}.java`
  (`recalculateBoundingBox`, `calculateBoundingBox`, yönün eklenme paketindeki verisi),
  `world/entity/projectile/arrow/{AbstractArrow,ThrownTrident}.java` (`tick`),
  `world/entity/projectile/ThrowableProjectile.java` (`tick`, `getDefaultGravity` ezmeleri).
- Rust: `crates/photon-world/src/pick.rs` (`bounding_box`), `crates/photon-world/src/lib.rs`
  (`block_attached_position`), `crates/photon-world/src/entity/projectile.rs`
  (`vanilla_projectile_kind`).

## Yapılacaklar
1. Asılı kutu: yön verisi (`ClientboundAddEntityPacket.getData`) ve haritalı çerçeve.
2. Mermi tiki: türlerin tik dalları; T47'nin `cull-trident` satırını üç tike çıkar,
   `cull-item-frame` ve `cull-glow-item-frame` satırlarını geri ekle.

## Oracle
`movement/living-entity-animations` (`cull` extra'sı kutuyu ve konumu taşır); gerekirse
eklenme yönü olayı.

## Kabul
`python3 tools/photon.py check --capture` → `CHECK OK`.

## Kapsam dışı
Tablo çizimi (T40h), çerçeve çizimi (T40g).

## Belgeler
`docs/records/client-entity-culling-parity-conformance.md` sapmalar maddesi.

## Devir notu
