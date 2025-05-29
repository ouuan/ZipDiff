package main

import (
    "os"
	"github.com/evilsocket/islazy/zip"
)

func main() {
	if _, err := zip.Unzip(os.Args[1], os.Args[2]); err != nil {
		panic(err)
	}
}
