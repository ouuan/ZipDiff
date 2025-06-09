package unzip;

import org.apache.commons.compress.archivers.zip.ZipArchiveEntry;
import org.apache.commons.compress.archivers.zip.ZipArchiveInputStream;

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
            ZipArchiveInputStream zipInputStream = new ZipArchiveInputStream(inputStream);
            ZipArchiveEntry entry;
            while ((entry = zipInputStream.getNextEntry()) != null) {
                File extractedFile = Paths.get(args[1], entry.getName()).toFile();
                if (entry.isDirectory()) {
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
