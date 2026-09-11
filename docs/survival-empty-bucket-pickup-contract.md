# P12 — Survival empty bucket pickup sözleşmesi

Bu dilim, survival oyuncunun slot 0'daki boş bucket'ı ayak hücresindeki water source üzerinde kullanmasını sınar. Amaç Java istemcisinin `use_item_on` → `use_item` fallback sırasını, source-only raycast'i, water source'un kaldırılmasını ve bucket'ın water bucket'a dönüşmesini karşılaştırmaktır.

## Java referansı

Empty bucket'ın item sınıfı block-use override'ı taşımadığı için Java istemcisi block hit üzerinde önce `use_item_on` yollar ve PASS sonucundan sonra `use_item` fallback'ine geçer. Server fallback action'ında bucket'ın source-only raycast'i water block'ı bulur. Empty-content bucket'ın doğrudan yerleştirme yolu başarısız olduktan sonra `BucketPickup` source'u kaldırır ve `ItemUtils` sonucu water bucket'a dönüştürür.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu console hazırlığı: oyuncunun ayak hücresine `minecraft:water`, slot 0'a `minecraft:bucket`
- İlk action: source block'ın üst yüzü, `use_item_on`, `MAIN_HAND`, sequence `25`
- Fallback action: `use_item`, `MAIN_HAND`, yRot `0.0`, xRot `90.0`, sequence `26`

Rust probe container content, slot, block update ve ACK packet'lerini toplar. Beklenen sonuç carried item packet sayısı `1`, her iki action packet sayısı `1`, ACK `[26]`, hazırlıkta slot `36` için bucket registry `1040`, kullanım sonrası content state `3` içinde slot `36` için water bucket registry `1041` ve source block için `state=86`dan `state=0`a geçiştir.

## Sınır

Bu dilim ayak hücresindeki source water pickup senaryosunu kapsar. Yana pickup, flowing water, lava/powder-snow bucket'ları, boş olmayan stack'ler, component patch'leri ve inventory click sonraki dilimlerdir.
