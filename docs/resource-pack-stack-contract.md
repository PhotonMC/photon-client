# P04.b — Pack kaynak önceliği sözleşmesi

Bu dilim, Java Edition 26.2 MultiPackResourceManager ve FallbackResourceManager için temel client-resource pack sırasını kapsar.

Pack listesi verilen sırayla namespace yöneticisine eklenir. Aynı identifier birden fazla pack'te varsa kaynak araması listenin sonundan yapılır ve son eşleşme seçilir. Kaynak stack, eşleşen pack'leri düşük öncelikliden yüksek öncelikliye doğru döndürür. Hiçbir pack'te bulunmayan identifier boş sonuç verir.

Rust üretim modülü pack kimliklerini ve sentetik kaynak byte'larını verilen sırada tutar; tekil lookup son pack'ten ilk eşleşmeyi, stack lookup ise giriş sırasını döndürür. Pack metadata/filter/overlay, namespace keşfi ve reload davranışı bu sözleşmeye dahil değildir.
