# T41g — Köylü, tüccar ve cadının tuttuğu ve taktığı eşya — [client-parity]

Bu kayıt, köylünün, gezgin tüccarın ve cadının tuttuğu eşyayı yerleştiren katmanların
(`CrossedArmsItemLayer`, `WitchItemLayer`) ve köylü/tüccarın kafa eşyası katmanının
(`CustomHeadLayer`, köylünün kendi dönüşümüyle) Java 26.2 ile Rust'ta aynı poz matrisini
üretmesini kapsar. Kanıt sınıfı `client-parity`'dir (dönüşümler) ve canlı gözlemdir (çizim):
gerçek katman sınıfları gerçek poz yığınında, bir eşya durumu gönderdiğinde yakalayıcı bir
`SubmitNodeCollector` ile okunur ve matris bit bit karşılaştırılır.

## Java kaynak incelemesi

- `HoldingEntityRenderState.heldItem` ana eldeki eşyayı `GROUND` bağlamında çözer; katman
  eşyayı bu bağlamın dönüşümüyle çizer (yere düşmüş görünümü).
- `CrossedArmsItemLayer`: modelin köke ve `arms` parçasına yürüyen dönüşümü, ardından 0,75
  radyan öne eğilme, 1,07 büyütme, `(0, 0,13, -0,34)` ötelemesi ve yarım turla çevirme.
  Bebek köylünün `arms` parçası da vardır, aynı formül.
- `WitchItemLayer`: iksirde kök, baş ve burnun kendi dönüşümleri, sonra `(0,0625, 0,25, 0)`
  ötelemesi ve z 180, x 140, z 10, x 180 derece; iksir değilse çapraz kol yolu.
  Elde bir şey varken burun öne çıkar (`setupAnim`, T34b).
- Kafa eşyası: köylüde `CUSTOM_HEAD_TRANSFORMS` (dikey kayma `-0,1171875`, yatay ölçek bir),
  tüccarda varsayılan; poz `head_item_pose` ile aynıdır.

## Rust

`photon-render/src/held_item.rs`: `crossed_arms_item_pose`, `witch_potion_pose` (kafa için
mevcut `head_item_pose`). `photon/src/held_items.rs`: `Place::Ground` (yere düşmüş eşya
bağlamlı önbellek) ve `HeldItem::stack`; `villagers.rs` eşyaları köylü, tüccar ve cadı için
toplar.

## Kapsam

`entity/villager-held-layers` 9 satır (çapraz kol: yetişkin ve bebek; cadı eşyası ve iki iksir
bakışı; kafa: köylü, mutsuz köylü, bebek köylü, tüccar).

## Sapmalar (açık)

- Zombi köylünün eşyası insansı tablonun eldeki eşya katmanından gelir (T28/T41); kafadaki
  eşyası zırh yolundadır.
- Eşyanın kendi model ağacı (özel modeller, bileşen yaması) mevcut `item_layers` yoludur.
