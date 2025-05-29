package unzip;

import net.lingala.zip4j.io.inputstream.ZipInputStream;
import net.lingala.zip4j.model.LocalFileHeader;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.file.Paths;

public class App {
    public static void main(String[] args) {
        try {
            InputStream inputStream = new FileInputStream(args[0]);
            ZipInputStream zipInputStream = new ZipInputStream(inputStream);
            LocalFileHeader localFileHeader;
            while ((localFileHeader = zipInputStream.getNextEntry()) != null) {
                File extractedFile = Paths.get(args[1], localFileHeader.getFileName()).toFile();
                if (localFileHeader.isDirectory()) {
                    extractedFile.mkdirs();
                } else {
                    extractedFile.getParentFile().mkdirs();
                    try (OutputStream outputStream = new FileOutputStream(extractedFile)) {
                        int readLen;
                        byte[] readBuffer = new byte[4096];
                        while ((readLen = zipInputStream.read(readBuffer)) != -1) {
                            outputStream.write(readBuffer, 0, readLen);
                        }
                    }
                }
            }
        } catch (IOException e) {
            e.printStackTrace();
            System.exit(1);
        }
    }
}
