# P04.d — Vanilla kaynak kökü sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Java `VanillaPackResourcesBuilder`, client-resource köklerini eklenme sırasını koruyarak toplar; pack oluşturulurken liste ters çevrilir ve kaynak araması son eklenen kökten başlar. `ClientPackSource` JAR kaynaklarını önce, haricî asset kökünü sonra eklediği için aynı identifier haricî kökte varsa haricî byte dizisi seçilir.

Vanilla pack'in namespace kümesi dosya köklerinden çıkarılmaz. Builder'a `exposeNamespace` ile verilen namespace'ler resource manager'a açılır; fiziksel kökte bulunan fakat açılmamış bir namespace manager sorgusunda görünmez. Açılmış namespace'te olmayan identifier boş sonuç verir.

Rust `VanillaPackResources` aynı kök sırasını ve açık namespace filtresini uygular. Pack'in manager içindeki dış kimliği `vanilla` olarak kalır; kök kimlikleri yalnızca iç seçim sırasını temsil eder.

Bu dilim JAR keşfinin classpath ayrıntılarını, development config callback'ini, `resourcepacks` klasörü keşfini, pack metadata'sını ve overlay/filter davranışını kapsamaz.


## Client-parity doğrulaması

Java oracle, `VanillaPackResourcesBuilder.pushAssetPath` ile iki sentetik client-resource kökü kurar ve bunları tek bir `MultiPackResourceManager` içindeki vanilla pack üzerinden sorgular. Köklerin aynı identifier'daki byte seçimi, yalnızca bir kökte bulunan kaynak, eksik kaynak ve `exposeNamespace` ile açılan namespace kümesi karşılaştırılır.

Kapsanan durumlar:

- son eklenen haricî kökün önceki kökteki aynı kaynağı geçmesi;
- yalnızca önceki veya yalnızca sonraki kökte bulunan kaynakların görünmesi;
- açılmış namespace'teki kaynakların görünmesi;
- fiziksel kökte bulunan fakat açılmamış namespace'in manager sorgusunda görünmemesi;
- eksik identifier için boş stack dönmesi.

Doğrulama Java 25.0.4.1 / Linux x86_64 üzerinde tamamlandı:

- 11 özgün kaynak senaryosu iki ayrı JVM çalıştırmasında kararlı bulundu;
- kaynak capture kimliği `ed86c0e1ff0b2ffe68d9f58730552b33dedbfb0c40c0494a8c55c6882c131222`;
- Java `verify` raporu `verify-qo0l6wy2`;
- Rust debug karşılaştırması `compare-rgokmott` ve release karşılaştırması `compare-nqg0pwif`, ikisi de fark üretmedi.

Yeni senaryolarla tam corpus doğrulaması da tamamlandı:

- 3407 senaryo için capture kimliği `d7d3e3c0109f40cc3c389b5a8fb53a4674c852414b3f3cc1eb3f233003bb68cc`;
- Java `verify` raporu `verify-cg4okiu_`;
- Rust debug/release karşılaştırmaları `compare-ah_5rkro` ve `compare-1r2v23n5`, ikisi de fark üretmedi.

JAR classpath keşfi, development config, haricî pack klasörü, metadata, filter ve overlay sonraki P04 dilimleridir.
