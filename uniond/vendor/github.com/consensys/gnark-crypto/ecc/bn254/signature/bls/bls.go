// Copyright 2020 ConsenSys Software Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Code generated by consensys/gnark-crypto DO NOT EDIT

package bls

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"crypto/sha512"
	"crypto/subtle"
	"errors"
	"hash"
	"io"
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark-crypto/signature"
)

var errHashNotNil = errors.New("hFunc not nil. Hashing the message is done in HashToG2.")

const (
	sizeFr         = fr.Bytes
	sizeFp         = fp.Bytes
	sizePublicKey  = sizeFp
	sizePrivateKey = sizeFr + sizePublicKey
	sizeSignature  = 2 * sizeFp
)

var order = fr.Modulus()

// PublicKey represents an BLS public key
type PublicKey struct {
	A bn254.G1Affine
}

// PrivateKey represents an BLS private key
type PrivateKey struct {
	PublicKey PublicKey
	scalar    [sizeFr]byte // secret scalar, in big Endian
}

// Signature represents an BLS signature
type Signature struct {
	S bn254.G2Affine
}

var one = new(big.Int).SetInt64(1)

// randFieldElement returns a random element of the order of the given
// curve using the procedure given in FIPS 186-4, Appendix B.5.1.
func randFieldElement(rand io.Reader) (k *big.Int, err error) {
	b := make([]byte, fr.Bits/8+8)
	_, err = io.ReadFull(rand, b)
	if err != nil {
		return
	}

	k = new(big.Int).SetBytes(b)
	n := new(big.Int).Sub(order, one)
	k.Mod(k, n)
	k.Add(k, one)
	return
}

// GenerateKey generates a public and private key pair.
func GenerateKey(rand io.Reader) (*PrivateKey, error) {

	k, err := randFieldElement(rand)
	if err != nil {
		return nil, err

	}
	_, _, g, _ := bn254.Generators()

	privateKey := new(PrivateKey)
	k.FillBytes(privateKey.scalar[:sizeFr])
	privateKey.PublicKey.A.ScalarMultiplication(&g, k)
	return privateKey, nil
}

type zr struct{}

// Read replaces the contents of dst with zeros. It is safe for concurrent use.
func (zr) Read(dst []byte) (n int, err error) {
	for i := range dst {
		dst[i] = 0
	}
	return len(dst), nil
}

var zeroReader = zr{}

const (
	aesIV = "gnark-crypto IV." // must be 16 chars (equal block size)
)

func nonce(privateKey *PrivateKey, hash []byte) (csprng *cipher.StreamReader, err error) {
	// This implementation derives the nonce from an AES-CTR CSPRNG keyed by:
	//
	//    SHA2-512(privateKey.scalar ∥ entropy ∥ hash)[:32]
	//
	// The CSPRNG key is indifferentiable from a random oracle as shown in
	// [Coron], the AES-CTR stream is indifferentiable from a random oracle
	// under standard cryptographic assumptions (see [Larsson] for examples).
	//
	// [Coron]: https://cs.nyu.edu/~dodis/ps/merkle.pdf
	// [Larsson]: https://web.archive.org/web/20040719170906/https://www.nada.kth.se/kurser/kth/2D1441/semteo03/lecturenotes/assump.pdf

	// Get 256 bits of entropy from rand.
	entropy := make([]byte, 32)
	_, err = io.ReadFull(rand.Reader, entropy)
	if err != nil {
		return

	}

	// Initialize an SHA-512 hash context; digest...
	md := sha512.New()
	md.Write(privateKey.scalar[:sizeFr]) // the private key,
	md.Write(entropy)                    // the entropy,
	md.Write(hash)                       // and the input hash;
	key := md.Sum(nil)[:32]              // and compute ChopMD-256(SHA-512),
	// which is an indifferentiable MAC.

	// Create an AES-CTR instance to use as a CSPRNG.
	block, _ := aes.NewCipher(key)

	// Create a CSPRNG that xors a stream of zeros with
	// the output of the AES-CTR instance.
	csprng = &cipher.StreamReader{
		R: zeroReader,
		S: cipher.NewCTR(block, []byte(aesIV)),
	}

	return csprng, err
}

// Equal compares 2 public keys
func (pub *PublicKey) Equal(x signature.PublicKey) bool {
	xx, ok := x.(*PublicKey)
	if !ok {
		return false
	}
	bpk := pub.Bytes()
	bxx := xx.Bytes()
	return subtle.ConstantTimeCompare(bpk, bxx) == 1
}

// Public returns the public key associated to the private key.
func (privKey *PrivateKey) Public() signature.PublicKey {
	var pub PublicKey
	pub.A.Set(&privKey.PublicKey.A)
	return &pub
}

// Sign performs the BLS signature
//
// H(m) ← m (hash to G₂)
// S = sk ⋅ H(m) ∈ G₂
// signature = S
func (privKey *PrivateKey) Sign(message []byte, hFunc hash.Hash) ([]byte, error) {

	// hFunc should always be nil.
	// Hashing the message first with hFunc is not needed, as it is included in HashToG2.
	if hFunc != nil {
		return nil, errHashNotNil
	}

	// Hash the message into G2
	dst := []byte("0x01")
	H, err := bn254.HashToG2(message, dst)
	if err != nil {
		return nil, err
	}

	// Sign
	var sig Signature
	scalar := new(big.Int)
	scalar.SetBytes(privKey.scalar[:sizeFr])
	sig.S.ScalarMultiplication(&H, scalar)

	return sig.Bytes(), nil
}

// Verify validates the BLS signature
//
// e(G1, sig.S) ?= e(pk, G2)
func (publicKey *PublicKey) Verify(sigBin, message []byte, hFunc hash.Hash) (bool, error) {

	// hFunc should always be nil.
	// Hashing the message first with hFunc is not needed, as it is included in HashToG2.
	if hFunc != nil {
		return false, errHashNotNil
	}

	// Deserialize the signature
	var sig Signature
	if _, err := sig.SetBytes(sigBin); err != nil {
		return false, err
	}
	// Hash the message into G2
	dst := []byte("0x01")
	H, err := bn254.HashToG2(message, dst)
	if err != nil {
		return false, err
	}

	// Verify the signature
	_, _, g1, _ := bn254.Generators()
	g1.Neg(&g1)
	f, err := bn254.PairingCheck([]bn254.G1Affine{g1, publicKey.A}, []bn254.G2Affine{sig.S, H})
	if err != nil {
		return false, err
	}

	return f, nil

}
