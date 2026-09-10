# P04.b — Known pack değer sözleşmesi

Java `KnownPack` üç string alanı (`namespace`, `id`, `version`) birlikte taşır. `KnownPack.vanilla(id)`, namespace'i `minecraft`, id'yi çağıran değeri ve sabitlenmiş client sürümünü (`26.2`) kullanır. `isVanilla` yalnızca namespace'in tam olarak `minecraft` olmasıyla belirlenir; id veya version ayrıca doğrulanmaz.

String gösterimi alanları `namespace:id:version` sırasıyla ve ayraçlarla birleştirir. Boş alanlar için ayraçlar korunur.

Rust `KnownPack` alanları, vanilla üreticisini, namespace karşılaştırmasını ve Display biçimini aynı şekilde taşır. Stream codec, network configuration içindeki known-pack listesi ve pack seçim sırası bu dilimin dışındadır.
