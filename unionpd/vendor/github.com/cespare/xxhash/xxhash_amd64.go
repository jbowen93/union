//go:build !appengine && gc && !purego
// +build !appengine,gc,!purego

package xxhash

// Sum64 computes the 64-bit xxHash digest of b.
//
//go:noescape
func Sum64(b []byte) uint64

func writeBlocks(x *xxh, b []byte) []byte
