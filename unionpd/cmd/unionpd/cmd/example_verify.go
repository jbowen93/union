package cmd

import (
	"context"
	"crypto/tls"
	"encoding/base64"
	"encoding/hex"
	"fmt"
	"log"
	"math/big"
	"time"
	provergrpc "unionp/grpc/api/v1"

	cometbft_bn254 "github.com/cometbft/cometbft/crypto/bn254"
	cometbn254 "github.com/cometbft/cometbft/crypto/bn254"
	ce "github.com/cometbft/cometbft/crypto/encoding"
	"github.com/cometbft/cometbft/crypto/merkle"
	"github.com/cometbft/cometbft/libs/protoio"
	"github.com/cometbft/cometbft/proto/tendermint/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/spf13/cobra"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/credentials/insecure"
)

// Example call to the prover `Prove` and then `Verify` endpoints using hardcoded values dumped from a local devnet.
// The sole purpose of this command is to see a live example and understand how to interact with the prover.
func ExampleVerifyCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:  "example-verify [uri]",
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {

			tlsEnabled, err := cmd.Flags().GetString(flagTLS)
			if err != nil {
				return err
			}

			var creds credentials.TransportCredentials
			if tlsEnabled == "yes" {
				creds = credentials.NewTLS(&tls.Config{})
			} else {
				creds = insecure.NewCredentials()
			}

			uri := args[0]
			conn, err := grpc.Dial(uri, grpc.WithTransportCredentials(creds))
			if err != nil {
				return err
			}
			defer conn.Close()
			client := provergrpc.NewUnionProverAPIClient(conn)
			ctx, cancel := context.WithTimeout(context.Background(), 1*time.Hour)
			defer cancel()

			// TODO: refactor: this code (prove call) is duplicated from `prove.go`
			decodeB64 := func(s string) []byte {
				bz, err := base64.StdEncoding.DecodeString(s)
				if err != nil {
					log.Fatal(err)
				}
				return bz
			}

			// Nb of tokens for each val in devnet
			tokens, success := new(big.Int).SetString("1000000000000000000000", 10)
			if !success {
				return fmt.Errorf("impossible; qed;")
			}

			toValidator := func(pubKey []byte) *types.SimpleValidator {
				protoPK, err := ce.PubKeyToProto(cometbn254.PubKey(pubKey))
				if err != nil {
					log.Fatal(err)
				}
				return &types.SimpleValidator{
					PubKey:      &protoPK,
					VotingPower: sdk.TokensToConsensusPower(sdk.NewIntFromBigInt(tokens), sdk.DefaultPowerReduction),
				}
			}

			blockHash, err := hex.DecodeString("1AD5BACC115AF66ADBA05C6D2393D73FD41E0DF1F761ED33344668BF71DEB9CB")
			if err != nil {
				return err
			}

			partSetHeaderHash, err := hex.DecodeString("6A80C88DA6FE1FA7773949270805567C963028008B10441E0180CF8AA1D400C9")
			if err != nil {
				return err
			}

			vote := types.CanonicalVote{
				Type:   types.PrecommitType,
				Height: 1,
				Round:  0,
				BlockID: &types.CanonicalBlockID{
					Hash: blockHash,
					PartSetHeader: types.CanonicalPartSetHeader{
						Total: 1,
						Hash:  partSetHeaderHash,
					},
				},
				ChainID: "union-devnet-1",
			}

			validators := []*types.SimpleValidator{
				toValidator(decodeB64("pNfYwyKvOhox3FNqU+ddZXqj8DS44ucdXs8mEfYPghI=")),
				toValidator(decodeB64("5vyjk9eK0ZsP06232NzpKp7dyz5AMmwG7sRHtje51pY=")),
				toValidator(decodeB64("hAPYPNTvyBT0Fl+BDrPlPFdWpq2eNI+YpHyEDaLpRGg=")),
				toValidator(decodeB64("nKwZsaaiIs/x+X+bOi+yPD2XR3Np3mf9iTYveD/JR3M=")),
			}

			trustedValidators := validators
			untrustedValidators := validators

			signatures := [][]byte{
				decodeB64("gdAsIuv3EMi250CS9dG6ym1exEAQm8gwYvJflmMDlroZiWIWI14nJhOHdXBqxevsjF1XInzck4sTsM8EuD3wJA=="),
				decodeB64("jtMDB9UOctP0tNloF/3RaPQXMYNadQt8T8DJYFgtHu8bC+9gpcyp7zcSc7OOrqQC8QKRGLBiGgX20F1BEQZLEw=="),
				decodeB64("ylbl7UYU2cBuaqxIFECloU+9yX2WAPGFXFkRt5Q7pg8ctKqz1Hz0oU7Fakyc/W+i6RDcFj9D+hpCWcx9HOEMiw=="),
				decodeB64("6lPTpzoSYY5N/F/TFUAGT+yyr3DOJV+Fq2JvCxOJojwOKe9e0bl+RB4ZarI9oB2YsQr/jLi2YfDLzo2tuvWfYw=="),
			}

			trustedSignatures := signatures
			untrustedSignatures := signatures

			var bitmap big.Int
			bitmap.SetBit(&bitmap, 0, 1)
			bitmap.SetBit(&bitmap, 1, 1)
			bitmap.SetBit(&bitmap, 2, 1)
			bitmap.SetBit(&bitmap, 3, 1)

			trustedBitmap := bitmap
			untrustedBitmap := bitmap

			proveRes, err := client.Prove(ctx, &provergrpc.ProveRequest{
				Vote: &vote,
				TrustedCommit: &provergrpc.ValidatorSetCommit{
					Validators: trustedValidators,
					Signatures: trustedSignatures,
					Bitmap:     trustedBitmap.Bytes(),
				},
				UntrustedCommit: &provergrpc.ValidatorSetCommit{
					Validators: untrustedValidators,
					Signatures: untrustedSignatures,
					Bitmap:     untrustedBitmap.Bytes(),
				},
			})
			if err != nil {
				log.Fatal(err)
			}

			log.Printf("Proof: %v\n", proveRes)

			trustedValidatorBytes := make([][]byte, len(trustedValidators))
			for i, val := range trustedValidators {
				protoEncoding, err := val.Marshal()
				if err != nil {
					return err
				}
				trustedValidatorBytes[i] = protoEncoding
			}

			untrustedValidatorBytes := make([][]byte, len(untrustedValidators))
			for i, val := range trustedValidators {
				protoEncoding, err := val.Marshal()
				if err != nil {
					return err
				}
				untrustedValidatorBytes[i] = protoEncoding
			}

			trustedValidatorSetRoot := merkle.HashFromByteSlices(trustedValidatorBytes)
			untrustedValidatorSetRoot := merkle.HashFromByteSlices(untrustedValidatorBytes)

			signedBytes, err := protoio.MarshalDelimited(&vote)
			if err != nil {
				return err
			}

			hmX, hmY := cometbft_bn254.HashToField2(signedBytes)

			verifyRes, err := client.Verify(ctx, &provergrpc.VerifyRequest{
				Proof:                     proveRes.Proof,
				TrustedValidatorSetRoot:   trustedValidatorSetRoot,
				UntrustedValidatorSetRoot: untrustedValidatorSetRoot,
				BlockHeaderX:              &provergrpc.FrElement{Value: hmX.Marshal()},
				BlockHeaderY:              &provergrpc.FrElement{Value: hmY.Marshal()},
			})

			if err != nil {
				log.Fatal(err)
			}

			log.Printf("Result: %v\n", verifyRes.Valid)

			return nil
		},
	}
	cmd.Flags().String(flagTLS, "", "Wether the gRPC endpoint expect TLS.")
	return cmd
}
