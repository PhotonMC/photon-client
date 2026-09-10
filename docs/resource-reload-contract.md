# P04.c — Kaynak reload sözleşmesi

`ReloadableResourceManager` boş bir aktif kaynak manager'ı ile başlar. Her reload çağrısı eski aktif pack stack'ini kapatır ve yeni pack listesinden bir manager kurar; reload tamamlandıktan sonra sorgular yalnızca yeni listedeki kaynakları görür.

Yeni pack listesi verilen sırayı korur. Aynı identifier birden fazla yeni pack'te varsa son pack seçilir ve kaynak stack'i düşük öncelikliden yüksek öncelikliye döner. Aktif namespace'ler yalnızca yeni pack listesindeki kaynaklardan oluşur; adapter çıktısında karşılaştırılabilirlik için sıralanır.

Bu dilim listener hazırlama sırası, executor zamanlaması, reload hataları, pack metadata/filter/overlay ve repository keşfini kapsamaz. Test girdileri özgün sentetik pack kimlikleri ve byte dizileridir.
