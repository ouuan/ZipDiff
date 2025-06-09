package unzip;

import java.io.File;
import org.codehaus.plexus.archiver.zip.ZipUnArchiver;

public class App {
    public static void main(String[] args) {
        var unarchiver = new ZipUnArchiver(new File(args[0]));
        unarchiver.setDestDirectory(new File(args[1]));
        unarchiver.extract();
    }
}
