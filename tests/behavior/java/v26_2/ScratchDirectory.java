import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * The scratch folders the oracle's rows make, all under one folder per run that is removed when
 * the run ends: a row used to leave its own folder behind, millions over the parity cases.
 */
final class ScratchDirectory {
    private static Path root;

    private ScratchDirectory() {}

    /** A new empty folder named after {@code prefix}, inside this run's folder. */
    static synchronized File make(String prefix) throws IOException {
        if (root == null) {
            root = Files.createTempDirectory("photon-oracle-run");
            var folder = root;
            Runtime.getRuntime().addShutdownHook(new Thread(() -> remove(folder.toFile())));
        }
        return Files.createTempDirectory(root, prefix).toFile();
    }

    private static void remove(File file) {
        var children = file.listFiles();
        if (children != null) {
            for (var child : children) {
                remove(child);
            }
        }
        file.delete();
    }
}
