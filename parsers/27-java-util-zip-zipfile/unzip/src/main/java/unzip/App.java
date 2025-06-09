package unzip;

import java.io.File;
import org.zeroturnaround.zip.ZipUtil;

public class App {
    public static void main(String[] args) {
        ZipUtil.unpack(new File(args[0]), new File(args[1]));
    }
}
