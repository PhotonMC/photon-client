# T25d — Yerdeki eşya akışkanda

**Amaç:** Suya/lava düşen eşya Java'daki gibi yüzer: `ItemEntity.tick`'in `setUnderwaterMovement`/`setUnderLavaMovement` kolları ve `Entity.updateFluidInteraction` akıntısı. T25c'de akışkana değen eşya simüle edilmiyor, sunucu konumunda kalıyor.
**Bağımlılık:** T25c. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/world/entity/item/ItemEntity.java` (`tick` 131–137, `setFluidMovement` 199–207), `world/entity/Entity.java` (`updateFluidInteraction` 1646–1670, `FluidInteraction` sınıfı: `grep -rn "class FluidInteraction"`).
- Rust: `crates/voxel-world/src/item_physics.rs` (`touches_fluid` yerine gerçek kol), `local_player.rs` `update_fluid_interaction`/`FluidTracker`/`apply_current` (oyuncuya bağlı; entity'lerle paylaşılacak biçimde `Level`'e çıkar).

## Yapılacaklar
1. Akışkan izleyicisini (yükseklik, akıntı) oyuncudan bağımsız işleve çıkar; oyuncu ve eşya aynı kodu çağırsın.
2. `ItemEntity.tick`: `isInWater && fluidHeight > 0.1` → `0.99`, lav `0.95` çarpanı ve `+5e-4` dikey itme; yoksa yerçekimi.
3. Oracle: `item_entity_ticks` satırları su/lav havuzu, akıntı ve yüzeyde duran eşya.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Kabarcık sütunu, blok içi etkiler (ağ vb.), dünya sınırı.

## Belgeler
`docs/client-world-entity-item-tick-parity-conformance.md` sapma listesini güncelle.

## Devir notu
(boş)
