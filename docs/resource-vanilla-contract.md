# P04.d — Vanilla kaynak kökü sözleşmesi

Java `VanillaPackResourcesBuilder`, client-resource köklerini eklenme sırasını koruyarak toplar; pack oluşturulurken liste ters çevrilir ve kaynak araması son eklenen kökten başlar. `ClientPackSource` JAR kaynaklarını önce, haricî asset kökünü sonra eklediği için aynı identifier haricî kökte varsa haricî byte dizisi seçilir.

Vanilla pack'in namespace kümesi dosya köklerinden çıkarılmaz. Builder'a `exposeNamespace` ile verilen namespace'ler resource manager'a açılır; fiziksel kökte bulunan fakat açılmamış bir namespace manager sorgusunda görünmez. Açılmış namespace'te olmayan identifier boş sonuç verir.

Rust `VanillaPackResources` aynı kök sırasını ve açık namespace filtresini uygular. Pack'in manager içindeki dış kimliği `vanilla` olarak kalır; kök kimlikleri yalnızca iç seçim sırasını temsil eder.

Bu dilim JAR keşfinin classpath ayrıntılarını, development config callback'ini, `resourcepacks` klasörü keşfini, pack metadata'sını ve overlay/filter davranışını kapsamaz.
