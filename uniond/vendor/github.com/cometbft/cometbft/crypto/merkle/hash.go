package merkle

import (
	"github.com/cometbft/cometbft/crypto/tmhash"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr/mimc"
)

// TODO: make these have a large predefined capacity
var (
	leafPrefix  = []byte{0}
	innerPrefix = []byte{1}
)

// returns tmhash(<empty>)
func emptyHash() []byte {
	return tmhash.Sum([]byte{})
}

// returns tmhash(0x00 || leaf)
func leafHash(leaf []byte) []byte {
	return tmhash.Sum(append(leafPrefix, leaf...))
}

// returns tmhash(0x01 || left || right)
func innerHash(left []byte, right []byte) []byte {
	return tmhash.Sum(append(innerPrefix, append(left, right...)...))
}

// returns mimc(<empty>)
func emptyMimcHash() []byte {
	return mimc.NewMiMC().Sum([]byte{})
}

// returns mimc(0x00 || leaf)
func leafMimcHash(leaf []byte) []byte {
	return mimc.NewMiMC().Sum(append(leafPrefix, leaf...))
}

// returns mimc(0x01 || left || right)
func innerMimcHash(left []byte, right []byte) []byte {
	return mimc.NewMiMC().Sum(append(innerPrefix, append(left, right...)...))
}
