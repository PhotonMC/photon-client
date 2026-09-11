# P12 — Survival water bucket use-on sözleşmesi

Bu dilim, survival oyuncunun slot 0'daki tek water bucket'ı destek bloğunun üst yüzünde kullanmasını sınar. Amaç Java istemcisinin `use_item_on` → `use_item` fallback sırasını, sequence ACK davranışını, suyun otoriter blok güncellemesini ve water bucket'ın bucket'a dönüşmesini karşılaştırmaktır.

## Java referansı

Java istemcisi block hit üzerinde önce `use_item_on` action'ını başlatır. Water bucket'ın item sınıfı block-use override'ı taşımadığı için bu action item tarafından tüketilmez; istemci aynı input işleminin devamında oyuncunun bakış rotasyonuyla `use_item` yollar. Server ikinci action'da sequence'i ACK kuyruğuna alır, oyuncunun POV raycast'ini kullanarak bucket içeriğini yerleştirir ve survival eldeki sonucu boş bucket'a dönüştürür. İlk listener snapshot'ı ile fluid yerleşiminin block update'i ayrı packet'ler olarak görülebilir.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: oyuncu oyuna girdikten sonra `item replace entity <player> hotbar.0 with minecraft:water_bucket`
- Ortam: oyuncunun altındaki y=`100` dirt destek bloğu; üst hücre başlangıçta air
- Rust hazırlığı: console komutları için yirmi saniyelik aralık, ardından `SET_CARRIED_ITEM`, slot `0`
- İlk action: destek bloğunun üst yüzü, `use_item_on`, `MAIN_HAND`, sequence `23`
- Fallback action: `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `90.0`, sequence `24`

Rust probe container content, slot, block update ve ACK packet'lerini toplar. Beklenen sonuç carried item packet sayısı `1`, `use_item_on` ve `use_item` sayıları `1`, ACK `[24]`, hazırlıkta slot `36` için water bucket registry `1041`, kullanım sonrası content state `3` içinde slot `36` için bucket registry `1040` ve üst hücrede water source state `86` gözlemidir.

## Sınır

Bu dilim üst yüze su kaynağı koyan tek water bucket senaryosunu kapsar. Yana yerleştirme, boş bucket ile fluid pickup, akışın bütün komşulara yayılması, lava/powder-snow bucket'ları, component patch'leri, farklı stack sayıları ve inventory click sonraki dilimlerdir.
