package lightclient

import (
	"fmt"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/test"
	"github.com/stretchr/testify/assert"
)

type UnpackRepackISO struct {
	Value frontend.Variable
}

func (c *UnpackRepackISO) Define(api frontend.API) error {
	api.AssertIsEqual(c.Value, Repack(api, Unpack(api, c.Value, 256, 1), 256, 256)[0])
	return nil
}

func TestUnpackRepackISO(t *testing.T) {
	t.Parallel()
	for i := 0; i < 100; i++ {
		k := i
		t.Run(fmt.Sprintf("%d", k), func(t *testing.T) {
			t.Parallel()
			var value fr.Element
			_, err := value.SetRandom()
			assert.NoError(t, err)
			test.NewAssert(t).CheckCircuit(
				&UnpackRepackISO{},
				test.WithValidAssignment(&UnpackRepackISO{
					Value: value,
				}),
				test.WithCurves(ecc.BN254),
				test.NoFuzzing(),
				test.WithCurves(ecc.BN254),
				test.WithBackends(backend.GROTH16),
			)
		})
	}
}
