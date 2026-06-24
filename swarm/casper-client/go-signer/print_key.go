package main

import (
	"fmt"
	"os"
	"strings"

	"github.com/make-software/casper-go-sdk/v2/types/keypair"
)

func main() {
	if len(os.Args) < 3 {
		fmt.Println("Usage: go run print_key.go <pem_path> <algo>")
		os.Exit(1)
	}
	pemPath := os.Args[1]
	algoStr := os.Args[2]

	pemAlgo := keypair.ED25519
	if strings.ToLower(algoStr) == "secp256k1" {
		pemAlgo = keypair.SECP256K1
	}

	privKey, err := keypair.NewPrivateKeyFromFile(pemPath, pemAlgo)
	if err != nil {
		fmt.Printf("Error loading private key: %v\n", err)
		os.Exit(1)
	}

	pubKey := privKey.PublicKey()
	fmt.Printf("Algorithm: %s\n", algoStr)
	fmt.Printf("PublicKey (hex): %s\n", pubKey.ToHex())
	fmt.Printf("Account Hash: %s\n", pubKey.AccountHash().ToHex())
}
