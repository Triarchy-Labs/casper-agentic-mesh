package main

import (
	"context"
	"flag"
	"fmt"
	"math/big"
	"net/http"
	"os"
	"strconv"
	"strings"
	"time"

	"github.com/make-software/casper-go-sdk/v2/casper"
	"github.com/make-software/casper-go-sdk/v2/types"
	"github.com/make-software/casper-go-sdk/v2/types/clvalue"
	"github.com/make-software/casper-go-sdk/v2/types/key"
	"github.com/make-software/casper-go-sdk/v2/types/keypair"
)

func main() {
	mode := flag.String("mode", "", "deploy-wasm or call-entrypoint")
	nodeURL := flag.String("node", "https://node.testnet.casper.network/rpc", "Casper RPC Node URL")
	chainName := flag.String("chain", "casper-test", "Casper Chain Name")
	secretKeyPath := flag.String("secret-key", "", "Path to secret_key.pem")
	algo := flag.String("algo", "ed25519", "ed25519 or secp256k1")
	paymentStr := flag.String("payment", "50000000000", "Payment amount in motes") // 50 CSPR default

	// Deploy WASM parameters
	wasmPath := flag.String("wasm", "", "Path to compiled .wasm contract")

	// Call Entrypoint parameters
	contractHashStr := flag.String("contract-hash", "", "Contract hash (hex with or without hash- prefix)")
	entrypoint := flag.String("entrypoint", "", "Contract entrypoint name")
	argsList := flag.String("args", "", "Comma-separated key:type:val arguments (e.g. 'amount:u512:1000,receiver:string:abc')")

	flag.Parse()

	if *mode == "" || *secretKeyPath == "" {
		fmt.Fprintf(os.Stderr, "Error: --mode and --secret-key are required\n")
		os.Exit(1)
	}

	paymentMotes, err := strconv.ParseUint(*paymentStr, 10, 64)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error parsing payment: %v\n", err)
		os.Exit(1)
	}

	// Load Private Key
	pemAlgo := keypair.ED25519
	if strings.ToLower(*algo) == "secp256k1" {
		pemAlgo = keypair.SECP256K1
	}

	privKey, err := keypair.NewPrivateKeyFromFile(*secretKeyPath, pemAlgo)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error loading private key: %v\n", err)
		os.Exit(1)
	}
	pubKey := privKey.PublicKey()

	// Initialize Casper client
	handler := casper.NewRPCHandler(*nodeURL, http.DefaultClient)
	client := casper.NewRPCClient(handler)

	var deploy *types.TransactionV1

	if *mode == "deploy-wasm" {
		if *wasmPath == "" {
			fmt.Fprintf(os.Stderr, "Error: --wasm path is required for deploy-wasm mode\n")
			os.Exit(1)
		}
		wasmBytes, err := os.ReadFile(*wasmPath)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error reading wasm: %v\n", err)
			os.Exit(1)
		}

		args := types.Args{}
		// Add default arguments if any
		namedArgs := types.NewNamedArgs(&args)

		v1Payload, err := types.NewTransactionV1Payload(
			types.InitiatorAddr{
				PublicKey: &pubKey,
			},
			types.Timestamp(time.Now().UTC()),
			900000000000, // ttl
			*chainName,
			types.PricingMode{
				Limited: &types.LimitedMode{
					GasPriceTolerance: 1,
					StandardPayment:   true,
					PaymentAmount:     paymentMotes,
				},
			},
			namedArgs,
			types.TransactionTarget{
				Session: &types.SessionTarget{
					IsInstallUpgrade: true,
					ModuleBytes:      wasmBytes,
					Runtime:          types.NewVmCasperV1TransactionRuntime(),
				},
			},
			types.TransactionEntryPoint{
				Call: &struct{}{},
			},
			types.TransactionScheduling{
				Standard: &struct{}{},
			},
		)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error creating payload: %v\n", err)
			os.Exit(1)
		}

		deploy, err = types.MakeTransactionV1(v1Payload)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error making transaction: %v\n", err)
			os.Exit(1)
		}

	} else if *mode == "session-wasm" {
		if *wasmPath == "" {
			fmt.Fprintf(os.Stderr, "Error: --wasm path is required for session-wasm mode\n")
			os.Exit(1)
		}
		wasmBytes, err := os.ReadFile(*wasmPath)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error reading wasm: %v\n", err)
			os.Exit(1)
		}

		args := types.Args{}
		if *argsList != "" {
			parts := strings.Split(*argsList, ",")
			for _, part := range parts {
				kvt := strings.Split(part, ":")
				if len(kvt) != 3 {
					fmt.Fprintf(os.Stderr, "Error: invalid argument format '%s', must be name:type:value\n", part)
					os.Exit(1)
				}
				argName, argType, argVal := kvt[0], kvt[1], kvt[2]
				switch strings.ToLower(argType) {
				case "string":
					args.AddArgument(argName, *clvalue.NewCLString(argVal))
				case "u512":
					valBig, ok := new(big.Int).SetString(argVal, 10)
					if !ok {
						fmt.Fprintf(os.Stderr, "Error: invalid U512 value '%s'\n", argVal)
						os.Exit(1)
					}
					args.AddArgument(argName, *clvalue.NewCLUInt512(valBig))
				case "uref":
					parsedURef, err := key.NewURef(argVal)
					if err != nil {
						fmt.Fprintf(os.Stderr, "Error: invalid URef value '%s': %v\n", argVal, err)
						os.Exit(1)
					}
					args.AddArgument(argName, clvalue.NewCLUref(parsedURef))
				default:
					fmt.Fprintf(os.Stderr, "Error: unsupported session arg type '%s'\n", argType)
					os.Exit(1)
				}
			}
		}
		namedArgs := types.NewNamedArgs(&args)

		v1Payload, err := types.NewTransactionV1Payload(
			types.InitiatorAddr{PublicKey: &pubKey},
			types.Timestamp(time.Now().UTC()),
			900000000000,
			*chainName,
			types.PricingMode{
				Limited: &types.LimitedMode{
					GasPriceTolerance: 1,
					StandardPayment:   true,
					PaymentAmount:     paymentMotes,
				},
			},
			namedArgs,
			types.TransactionTarget{
				Session: &types.SessionTarget{
					IsInstallUpgrade: false,
					ModuleBytes:      wasmBytes,
					Runtime:          types.NewVmCasperV1TransactionRuntime(),
				},
			},
			types.TransactionEntryPoint{
				Call: &struct{}{},
			},
			types.TransactionScheduling{
				Standard: &struct{}{},
			},
		)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error creating payload: %v\n", err)
			os.Exit(1)
		}
		deploy, err = types.MakeTransactionV1(v1Payload)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error making transaction: %v\n", err)
			os.Exit(1)
		}

	} else if *mode == "call-entrypoint" {
		if *contractHashStr == "" || *entrypoint == "" {
			fmt.Fprintf(os.Stderr, "Error: --contract-hash and --entrypoint are required\n")
			os.Exit(1)
		}

		cleanedHash := strings.TrimPrefix(*contractHashStr, "hash-")
		cleanedHash = strings.TrimPrefix(cleanedHash, "contract-")
		contractHash, err := casper.NewHash(cleanedHash)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error parsing contract hash: %v\n", err)
			os.Exit(1)
		}

		args := types.Args{}
		if *argsList != "" {
			parts := strings.Split(*argsList, ",")
			for _, part := range parts {
				kvt := strings.Split(part, ":")
				if len(kvt) != 3 {
					fmt.Fprintf(os.Stderr, "Error: invalid argument format '%s', must be name:type:value\n", part)
					os.Exit(1)
				}
				argName := kvt[0]
				argType := kvt[1]
				argVal := kvt[2]

				switch strings.ToLower(argType) {
				case "string":
					args.AddArgument(argName, *clvalue.NewCLString(argVal))
				case "u512":
					valBig, ok := new(big.Int).SetString(argVal, 10)
					if !ok {
						fmt.Fprintf(os.Stderr, "Error: invalid U512 value '%s'\n", argVal)
						os.Exit(1)
					}
					args.AddArgument(argName, *clvalue.NewCLUInt512(valBig))
				case "key":
					// Try account hash
					var parsedKey key.Key
					if strings.HasPrefix(argVal, "account-hash-") {
						parsedKey, err = key.NewKey(argVal)
					} else {
						parsedKey, err = key.NewKey("account-hash-" + argVal)
					}
					if err != nil {
						// Try contract hash
						if strings.HasPrefix(argVal, "hash-") {
							parsedKey, err = key.NewKey(argVal)
						} else {
							parsedKey, err = key.NewKey("hash-" + argVal)
						}
					}
					if err != nil {
						fmt.Fprintf(os.Stderr, "Error: invalid Key value '%s': %v\n", argVal, err)
						os.Exit(1)
					}
					args.AddArgument(argName, clvalue.NewCLKey(parsedKey))
				case "uref":
					parsedURef, err := key.NewURef(argVal)
					if err != nil {
						fmt.Fprintf(os.Stderr, "Error: invalid URef value '%s': %v\n", argVal, err)
						os.Exit(1)
					}
					args.AddArgument(argName, clvalue.NewCLUref(parsedURef))
				default:
					fmt.Fprintf(os.Stderr, "Error: unsupported argument type '%s'\n", argType)
					os.Exit(1)
				}
			}
		}

		namedArgs := types.NewNamedArgs(&args)

		v1Payload, err := types.NewTransactionV1Payload(
			types.InitiatorAddr{
				PublicKey: &pubKey,
			},
			types.Timestamp(time.Now().UTC()),
			900000000000,
			*chainName,
			types.PricingMode{
				Limited: &types.LimitedMode{
					GasPriceTolerance: 1,
					StandardPayment:   true,
					PaymentAmount:     paymentMotes,
				},
			},
			namedArgs,
			types.TransactionTarget{
				Stored: &types.StoredTarget{
					ID: types.TransactionInvocationTarget{
						ByPackageHash: &types.ByPackageHashInvocationTarget{
							Addr:    contractHash,
							Version: nil,
						},
					},
					Runtime: types.NewVmCasperV1TransactionRuntime(),
				},
			},
			types.TransactionEntryPoint{
				Custom: entrypoint,
			},
			types.TransactionScheduling{
				Standard: &struct{}{},
			},
		)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error creating payload: %v\n", err)
			os.Exit(1)
		}

		deploy, err = types.MakeTransactionV1(v1Payload)
		if err != nil {
			fmt.Fprintf(os.Stderr, "Error making transaction: %v\n", err)
			os.Exit(1)
		}
	} else {
		fmt.Fprintf(os.Stderr, "Error: unknown mode '%s'\n", *mode)
		os.Exit(1)
	}

	// Sign the transaction
	err = deploy.Sign(privKey)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error signing transaction: %v\n", err)
		os.Exit(1)
	}

	// Put transaction
	ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
	defer cancel()

	result, err := client.PutTransactionV1(ctx, *deploy)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Error submitting transaction: %v\n", err)
		os.Exit(1)
	}

	// Print transaction hash to stdout
	fmt.Printf("%s\n", result.TransactionHash.String())
}
