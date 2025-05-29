package unzip;

import java.io.File;
import java.io.FileInputStream;
import org.zeroturnaround.zip.ZipUtil;

public class App {
    public static void main(String[] args) {
        try {
            ZipUtil.unpack(new FileInputStream(new File(args[0])), new File(args[1]));
        } catch (Exception e) {
            e.printStackTrace();
            System.exit(1);
        }
    }
}
