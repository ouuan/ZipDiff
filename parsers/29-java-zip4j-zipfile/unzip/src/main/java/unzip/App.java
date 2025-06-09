package unzip;

import net.lingala.zip4j.ZipFile;
import net.lingala.zip4j.exception.ZipException;

public class App {
    public static void main(String[] args) {
        try {
            new ZipFile(args[0]).extractAll(args[1]);
        } catch (ZipException e) {
            System.err.println("Error during extraction: " + e.getMessage());
            System.exit(1);
        }
    }
}
