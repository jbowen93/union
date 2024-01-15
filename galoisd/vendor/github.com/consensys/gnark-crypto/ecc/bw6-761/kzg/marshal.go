// Copyright 2020 Consensys Software Inc.
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

package kzg

import (
	"github.com/consensys/gnark-crypto/ecc/bw6-761"
	"io"
)

// WriteTo writes binary encoding of the ProvingKey
func (pk *ProvingKey) WriteTo(w io.Writer) (int64, error) {
	return pk.writeTo(w)
}

// WriteRawTo writes binary encoding of ProvingKey to w without point compression
func (pk *ProvingKey) WriteRawTo(w io.Writer) (int64, error) {
	return pk.writeTo(w, bw6761.RawEncoding())
}

func (pk *ProvingKey) writeTo(w io.Writer, options ...func(*bw6761.Encoder)) (int64, error) {
	// encode the ProvingKey
	enc := bw6761.NewEncoder(w, options...)
	if err := enc.Encode(pk.G1); err != nil {
		return enc.BytesWritten(), err
	}
	return enc.BytesWritten(), nil
}

// WriteRawTo writes binary encoding of VerifyingKey to w without point compression
func (vk *VerifyingKey) WriteRawTo(w io.Writer) (int64, error) {
	return vk.writeTo(w, bw6761.RawEncoding())
}

// WriteTo writes binary encoding of the VerifyingKey
func (vk *VerifyingKey) WriteTo(w io.Writer) (int64, error) {
	return vk.writeTo(w)
}

func (vk *VerifyingKey) writeTo(w io.Writer, options ...func(*bw6761.Encoder)) (int64, error) {
	// encode the VerifyingKey
	enc := bw6761.NewEncoder(w, options...)
	nLines := 189
	toEncode := make([]interface{}, 0, 4*nLines+3)
	toEncode = append(toEncode, &vk.G2[0])
	toEncode = append(toEncode, &vk.G2[1])
	toEncode = append(toEncode, &vk.G1)
	for k := 0; k < 2; k++ {
		for j := 0; j < 2; j++ {
			for i := nLines - 1; i >= 0; i-- {
				toEncode = append(toEncode, &vk.Lines[k][j][i].R0)
				toEncode = append(toEncode, &vk.Lines[k][j][i].R1)
			}
		}
	}

	for _, v := range toEncode {
		if err := enc.Encode(v); err != nil {
			return enc.BytesWritten(), err
		}
	}

	return enc.BytesWritten(), nil
}

// WriteTo writes binary encoding of the entire SRS
func (srs *SRS) WriteTo(w io.Writer) (int64, error) {
	// encode the SRS
	var pn, vn int64
	var err error
	if pn, err = srs.Pk.WriteTo(w); err != nil {
		return pn, err
	}
	vn, err = srs.Vk.WriteTo(w)
	return pn + vn, err
}

// WriteRawTo writes binary encoding of the entire SRS without point compression
func (srs *SRS) WriteRawTo(w io.Writer) (int64, error) {
	// encode the SRS
	var pn, vn int64
	var err error
	if pn, err = srs.Pk.WriteRawTo(w); err != nil {
		return pn, err
	}
	vn, err = srs.Vk.WriteRawTo(w)
	return pn + vn, err
}

// ReadFrom decodes ProvingKey data from reader.
func (pk *ProvingKey) ReadFrom(r io.Reader) (int64, error) {
	// decode the ProvingKey
	dec := bw6761.NewDecoder(r)
	if err := dec.Decode(&pk.G1); err != nil {
		return dec.BytesRead(), err
	}
	return dec.BytesRead(), nil
}

// UnsafeReadFrom decodes ProvingKey data from reader without checking
// that point are in the correct subgroup.
func (pk *ProvingKey) UnsafeReadFrom(r io.Reader) (int64, error) {
	// decode the ProvingKey
	dec := bw6761.NewDecoder(r, bw6761.NoSubgroupChecks())
	if err := dec.Decode(&pk.G1); err != nil {
		return dec.BytesRead(), err
	}
	return dec.BytesRead(), nil
}

// ReadFrom decodes VerifyingKey data from reader.
func (vk *VerifyingKey) ReadFrom(r io.Reader) (int64, error) {
	// decode the VerifyingKey
	dec := bw6761.NewDecoder(r)
	nLines := 189
	toDecode := make([]interface{}, 0, 4*nLines+3)
	toDecode = append(toDecode, &vk.G2[0])
	toDecode = append(toDecode, &vk.G2[1])
	toDecode = append(toDecode, &vk.G1)
	for k := 0; k < 2; k++ {
		for j := 0; j < 2; j++ {
			for i := nLines - 1; i >= 0; i-- {
				toDecode = append(toDecode, &vk.Lines[k][j][i].R0)
				toDecode = append(toDecode, &vk.Lines[k][j][i].R1)
			}
		}
	}

	for _, v := range toDecode {
		if err := dec.Decode(v); err != nil {
			return dec.BytesRead(), err
		}
	}

	return dec.BytesRead(), nil
}

// ReadFrom decodes SRS data from reader.
func (srs *SRS) ReadFrom(r io.Reader) (int64, error) {
	// decode the VerifyingKey
	var pn, vn int64
	var err error
	if pn, err = srs.Pk.ReadFrom(r); err != nil {
		return pn, err
	}
	vn, err = srs.Vk.ReadFrom(r)
	return pn + vn, err
}

// UnsafeReadFrom decodes SRS data from reader without sub group checks
func (srs *SRS) UnsafeReadFrom(r io.Reader) (int64, error) {
	// decode the VerifyingKey
	var pn, vn int64
	var err error
	if pn, err = srs.Pk.UnsafeReadFrom(r); err != nil {
		return pn, err
	}
	vn, err = srs.Vk.ReadFrom(r)
	return pn + vn, err
}

// WriteTo writes binary encoding of a OpeningProof
func (proof *OpeningProof) WriteTo(w io.Writer) (int64, error) {
	enc := bw6761.NewEncoder(w)

	toEncode := []interface{}{
		&proof.H,
		&proof.ClaimedValue,
	}

	for _, v := range toEncode {
		if err := enc.Encode(v); err != nil {
			return enc.BytesWritten(), err
		}
	}

	return enc.BytesWritten(), nil
}

// ReadFrom decodes OpeningProof data from reader.
func (proof *OpeningProof) ReadFrom(r io.Reader) (int64, error) {
	dec := bw6761.NewDecoder(r)

	toDecode := []interface{}{
		&proof.H,
		&proof.ClaimedValue,
	}

	for _, v := range toDecode {
		if err := dec.Decode(v); err != nil {
			return dec.BytesRead(), err
		}
	}

	return dec.BytesRead(), nil
}

// WriteTo writes binary encoding of a BatchOpeningProof
func (proof *BatchOpeningProof) WriteTo(w io.Writer) (int64, error) {
	enc := bw6761.NewEncoder(w)

	toEncode := []interface{}{
		&proof.H,
		proof.ClaimedValues,
	}

	for _, v := range toEncode {
		if err := enc.Encode(v); err != nil {
			return enc.BytesWritten(), err
		}
	}

	return enc.BytesWritten(), nil
}

// ReadFrom decodes BatchOpeningProof data from reader.
func (proof *BatchOpeningProof) ReadFrom(r io.Reader) (int64, error) {
	dec := bw6761.NewDecoder(r)
	toDecode := []interface{}{
		&proof.H,
		&proof.ClaimedValues,
	}

	for _, v := range toDecode {
		if err := dec.Decode(v); err != nil {
			return dec.BytesRead(), err
		}
	}

	return dec.BytesRead(), nil
}
