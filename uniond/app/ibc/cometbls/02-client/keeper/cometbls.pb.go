//lint:file-ignore SA1019 This code is generated
// Code generated by protoc-gen-gogo. DO NOT EDIT.
// source: union/ibc/lightclients/cometbls/v1/cometbls.proto

package keeper

import (
	fmt "fmt"
	types2 "github.com/cometbft/cometbft/proto/tendermint/types"
	_ "github.com/cosmos/gogoproto/gogoproto"
	proto "github.com/cosmos/gogoproto/proto"
	types "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	types1 "github.com/cosmos/ibc-go/v8/modules/core/23-commitment/types"
	io "io"
	math "math"
	math_bits "math/bits"
)

// Reference imports to suppress errors if they are not otherwise used.
var _ = proto.Marshal
var _ = fmt.Errorf
var _ = math.Inf

// This is a compile-time assertion to ensure that this generated file
// is compatible with the proto package it is being compiled against.
// A compilation error at this line likely means your copy of the
// proto package needs to be updated.
const _ = proto.GoGoProtoPackageIsVersion3 // please upgrade the proto package

type ClientState struct {
	ChainId string `protobuf:"bytes,1,opt,name=chain_id,json=chainId,proto3" json:"chain_id,omitempty"`
	// duration of the period since the LastestTimestamp during which the
	// submitted headers are valid for upgrade
	TrustingPeriod uint64 `protobuf:"varint,2,opt,name=trusting_period,json=trustingPeriod,proto3" json:"trusting_period,omitempty"`
	// duration of the staking unbonding period
	UnbondingPeriod uint64 `protobuf:"varint,3,opt,name=unbonding_period,json=unbondingPeriod,proto3" json:"unbonding_period,omitempty"`
	// defines how much new (untrusted) header's Time can drift into the future.
	MaxClockDrift uint64 `protobuf:"varint,4,opt,name=max_clock_drift,json=maxClockDrift,proto3" json:"max_clock_drift,omitempty"`
	// Block height when the client was frozen due to a misbehaviour
	FrozenHeight types.Height `protobuf:"bytes,5,opt,name=frozen_height,json=frozenHeight,proto3" json:"frozen_height"`
	// Latest height the client was updated to
	LatestHeight types.Height `protobuf:"bytes,6,opt,name=latest_height,json=latestHeight,proto3" json:"latest_height"`
}

func (m *ClientState) Reset()         { *m = ClientState{} }
func (m *ClientState) String() string { return proto.CompactTextString(m) }
func (*ClientState) ProtoMessage()    {}
func (*ClientState) Descriptor() ([]byte, []int) {
	return fileDescriptor_6e4c33c744877a4e, []int{0}
}
func (m *ClientState) XXX_Unmarshal(b []byte) error {
	return m.Unmarshal(b)
}
func (m *ClientState) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	if deterministic {
		return xxx_messageInfo_ClientState.Marshal(b, m, deterministic)
	} else {
		b = b[:cap(b)]
		n, err := m.MarshalToSizedBuffer(b)
		if err != nil {
			return nil, err
		}
		return b[:n], nil
	}
}
func (m *ClientState) XXX_Merge(src proto.Message) {
	xxx_messageInfo_ClientState.Merge(m, src)
}
func (m *ClientState) XXX_Size() int {
	return m.Size()
}
func (m *ClientState) XXX_DiscardUnknown() {
	xxx_messageInfo_ClientState.DiscardUnknown(m)
}

var xxx_messageInfo_ClientState proto.InternalMessageInfo

type ConsensusState struct {
	// timestamp that corresponds to the block height in which the ConsensusState
	// was stored.
	Timestamp uint64 `protobuf:"varint,1,opt,name=timestamp,proto3" json:"timestamp,omitempty"`
	// commitment root (i.e app hash)
	Root               types1.MerkleRoot `protobuf:"bytes,2,opt,name=root,proto3" json:"root"`
	NextValidatorsHash []byte            `protobuf:"bytes,3,opt,name=next_validators_hash,json=nextValidatorsHash,proto3" json:"next_validators_hash,omitempty"`
}

func (m *ConsensusState) Reset()         { *m = ConsensusState{} }
func (m *ConsensusState) String() string { return proto.CompactTextString(m) }
func (*ConsensusState) ProtoMessage()    {}
func (*ConsensusState) Descriptor() ([]byte, []int) {
	return fileDescriptor_6e4c33c744877a4e, []int{1}
}
func (m *ConsensusState) XXX_Unmarshal(b []byte) error {
	return m.Unmarshal(b)
}
func (m *ConsensusState) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	if deterministic {
		return xxx_messageInfo_ConsensusState.Marshal(b, m, deterministic)
	} else {
		b = b[:cap(b)]
		n, err := m.MarshalToSizedBuffer(b)
		if err != nil {
			return nil, err
		}
		return b[:n], nil
	}
}
func (m *ConsensusState) XXX_Merge(src proto.Message) {
	xxx_messageInfo_ConsensusState.Merge(m, src)
}
func (m *ConsensusState) XXX_Size() int {
	return m.Size()
}
func (m *ConsensusState) XXX_DiscardUnknown() {
	xxx_messageInfo_ConsensusState.DiscardUnknown(m)
}

var xxx_messageInfo_ConsensusState proto.InternalMessageInfo

type Misbehaviour struct {
	Header_1 *Header `protobuf:"bytes,1,opt,name=header_1,json=header1,proto3" json:"header_1,omitempty"`
	Header_2 *Header `protobuf:"bytes,2,opt,name=header_2,json=header2,proto3" json:"header_2,omitempty"`
}

func (m *Misbehaviour) Reset()         { *m = Misbehaviour{} }
func (m *Misbehaviour) String() string { return proto.CompactTextString(m) }
func (*Misbehaviour) ProtoMessage()    {}
func (*Misbehaviour) Descriptor() ([]byte, []int) {
	return fileDescriptor_6e4c33c744877a4e, []int{2}
}
func (m *Misbehaviour) XXX_Unmarshal(b []byte) error {
	return m.Unmarshal(b)
}
func (m *Misbehaviour) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	if deterministic {
		return xxx_messageInfo_Misbehaviour.Marshal(b, m, deterministic)
	} else {
		b = b[:cap(b)]
		n, err := m.MarshalToSizedBuffer(b)
		if err != nil {
			return nil, err
		}
		return b[:n], nil
	}
}
func (m *Misbehaviour) XXX_Merge(src proto.Message) {
	xxx_messageInfo_Misbehaviour.Merge(m, src)
}
func (m *Misbehaviour) XXX_Size() int {
	return m.Size()
}
func (m *Misbehaviour) XXX_DiscardUnknown() {
	xxx_messageInfo_Misbehaviour.DiscardUnknown(m)
}

var xxx_messageInfo_Misbehaviour proto.InternalMessageInfo

func (m *Misbehaviour) GetHeader_1() *Header {
	if m != nil {
		return m.Header_1
	}
	return nil
}

func (m *Misbehaviour) GetHeader_2() *Header {
	if m != nil {
		return m.Header_2
	}
	return nil
}

type Header struct {
	SignedHeader       *types2.SignedHeader `protobuf:"bytes,1,opt,name=signed_header,json=signedHeader,proto3" json:"signed_header,omitempty"`
	TrustedHeight      *types.Height        `protobuf:"bytes,2,opt,name=trusted_height,json=trustedHeight,proto3" json:"trusted_height,omitempty"`
	ZeroKnowledgeProof []byte               `protobuf:"bytes,3,opt,name=zero_knowledge_proof,json=zeroKnowledgeProof,proto3" json:"zero_knowledge_proof,omitempty"`
}

func (m *Header) Reset()         { *m = Header{} }
func (m *Header) String() string { return proto.CompactTextString(m) }
func (*Header) ProtoMessage()    {}
func (*Header) Descriptor() ([]byte, []int) {
	return fileDescriptor_6e4c33c744877a4e, []int{3}
}
func (m *Header) XXX_Unmarshal(b []byte) error {
	return m.Unmarshal(b)
}
func (m *Header) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	if deterministic {
		return xxx_messageInfo_Header.Marshal(b, m, deterministic)
	} else {
		b = b[:cap(b)]
		n, err := m.MarshalToSizedBuffer(b)
		if err != nil {
			return nil, err
		}
		return b[:n], nil
	}
}
func (m *Header) XXX_Merge(src proto.Message) {
	xxx_messageInfo_Header.Merge(m, src)
}
func (m *Header) XXX_Size() int {
	return m.Size()
}
func (m *Header) XXX_DiscardUnknown() {
	xxx_messageInfo_Header.DiscardUnknown(m)
}

var xxx_messageInfo_Header proto.InternalMessageInfo

func (m *Header) GetSignedHeader() *types2.SignedHeader {
	if m != nil {
		return m.SignedHeader
	}
	return nil
}

func (m *Header) GetTrustedHeight() *types.Height {
	if m != nil {
		return m.TrustedHeight
	}
	return nil
}

func (m *Header) GetZeroKnowledgeProof() []byte {
	if m != nil {
		return m.ZeroKnowledgeProof
	}
	return nil
}

func init() {
	proto.RegisterType((*ClientState)(nil), "union.ibc.lightclients.cometbls.v1.ClientState")
	proto.RegisterType((*ConsensusState)(nil), "union.ibc.lightclients.cometbls.v1.ConsensusState")
	proto.RegisterType((*Misbehaviour)(nil), "union.ibc.lightclients.cometbls.v1.Misbehaviour")
	proto.RegisterType((*Header)(nil), "union.ibc.lightclients.cometbls.v1.Header")
}

func init() {
	proto.RegisterFile("union/ibc/lightclients/cometbls/v1/cometbls.proto", fileDescriptor_6e4c33c744877a4e)
}

var fileDescriptor_6e4c33c744877a4e = []byte{
	// 609 bytes of a gzipped FileDescriptorProto
	0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0x9c, 0x54, 0x3f, 0x6f, 0xd4, 0x3e,
	0x18, 0xbe, 0xf4, 0x77, 0xbf, 0xfe, 0xf1, 0xdd, 0xb5, 0x28, 0xea, 0x70, 0x54, 0x55, 0x5a, 0xdd,
	0x40, 0x0b, 0x12, 0x49, 0x13, 0x36, 0xc4, 0xd2, 0x1e, 0x95, 0x8a, 0x50, 0xa5, 0x2a, 0x95, 0x18,
	0x58, 0x22, 0x27, 0x79, 0x9b, 0x58, 0x97, 0xd8, 0x91, 0xed, 0x0b, 0xa5, 0x9f, 0x80, 0x91, 0x0f,
	0xc0, 0xc0, 0xc0, 0x87, 0x60, 0x66, 0xea, 0xd8, 0x91, 0x09, 0xa1, 0xde, 0x17, 0x41, 0xb1, 0x9d,
	0xbb, 0xdb, 0x8a, 0x58, 0x22, 0xfb, 0x79, 0x9f, 0xe7, 0xb1, 0xf3, 0xd8, 0xaf, 0x91, 0x3f, 0xa5,
	0x84, 0x51, 0x8f, 0xc4, 0x89, 0x57, 0x90, 0x2c, 0x97, 0x49, 0x41, 0x80, 0x4a, 0xe1, 0x25, 0xac,
	0x04, 0x19, 0x17, 0xc2, 0xab, 0xfd, 0xf9, 0xd8, 0xad, 0x38, 0x93, 0xcc, 0x1e, 0x29, 0x89, 0x4b,
	0xe2, 0xc4, 0x5d, 0x96, 0xb8, 0x73, 0x5a, 0xed, 0xef, 0xec, 0x4a, 0xa0, 0x29, 0xf0, 0x92, 0x50,
	0xe9, 0xc9, 0x8f, 0x15, 0x08, 0xfd, 0xd5, 0x0e, 0x3b, 0x7b, 0xcd, 0x72, 0x09, 0xe3, 0xe0, 0x69,
	0xad, 0x5a, 0x44, 0x8d, 0x0c, 0xe1, 0x60, 0x41, 0x60, 0x65, 0x49, 0x64, 0xd9, 0x92, 0xe6, 0x33,
	0x43, 0xdc, 0xce, 0x58, 0xc6, 0xd4, 0xd0, 0x6b, 0x46, 0x1a, 0x1d, 0x7d, 0x5f, 0x41, 0xbd, 0xb1,
	0xf2, 0xbb, 0x94, 0x58, 0x82, 0xfd, 0x18, 0xad, 0x27, 0x39, 0x26, 0x34, 0x22, 0xe9, 0xd0, 0xda,
	0xb7, 0x0e, 0x37, 0xc2, 0x35, 0x35, 0x7f, 0x93, 0xda, 0x07, 0x68, 0x4b, 0xf2, 0xa9, 0x90, 0x84,
	0x66, 0x51, 0x05, 0x9c, 0xb0, 0x74, 0xb8, 0xb2, 0x6f, 0x1d, 0x76, 0xc3, 0xcd, 0x16, 0xbe, 0x50,
	0xa8, 0xfd, 0x14, 0x3d, 0x9a, 0xd2, 0x98, 0xd1, 0x74, 0x89, 0xf9, 0x9f, 0x62, 0x6e, 0xcd, 0x71,
	0x43, 0x7d, 0x82, 0xb6, 0x4a, 0x7c, 0x1d, 0x25, 0x05, 0x4b, 0x26, 0x51, 0xca, 0xc9, 0x95, 0x1c,
	0x76, 0x15, 0x73, 0x50, 0xe2, 0xeb, 0x71, 0x83, 0xbe, 0x6e, 0x40, 0xfb, 0x14, 0x0d, 0xae, 0x38,
	0xbb, 0x01, 0x1a, 0xe5, 0xd0, 0x04, 0x39, 0xfc, 0x7f, 0xdf, 0x3a, 0xec, 0x05, 0x3b, 0x2a, 0xda,
	0xe6, 0xef, 0x5d, 0x13, 0x4a, 0xed, 0xbb, 0x67, 0x8a, 0x71, 0xd2, 0xbd, 0xfd, 0xb5, 0xd7, 0x09,
	0xfb, 0x5a, 0xa6, 0xb1, 0xc6, 0xa6, 0xc0, 0x12, 0x84, 0x6c, 0x6d, 0x56, 0xff, 0xd6, 0x46, 0xcb,
	0x34, 0xf6, 0xb2, 0xfb, 0xe9, 0xeb, 0x5e, 0x67, 0xf4, 0xcd, 0x42, 0x9b, 0x63, 0x46, 0x05, 0x50,
	0x31, 0x15, 0x3a, 0xbd, 0x5d, 0xb4, 0x21, 0x49, 0x09, 0x42, 0xe2, 0xb2, 0x52, 0xf1, 0x75, 0xc3,
	0x05, 0x60, 0xbf, 0x42, 0x5d, 0xce, 0x98, 0x54, 0xa9, 0xf5, 0x82, 0xd1, 0xd2, 0xa2, 0x8b, 0xb3,
	0xaa, 0x7d, 0xf7, 0x1c, 0xf8, 0xa4, 0x80, 0x90, 0xb1, 0x76, 0x71, 0xa5, 0xb2, 0x8f, 0xd0, 0x36,
	0x85, 0x6b, 0x19, 0xd5, 0xb8, 0x20, 0x29, 0x96, 0x8c, 0x8b, 0x28, 0xc7, 0x22, 0x57, 0xc9, 0xf6,
	0x43, 0xbb, 0xa9, 0xbd, 0x9b, 0x97, 0xce, 0xb0, 0xc8, 0xcd, 0x36, 0xbf, 0x58, 0xa8, 0x7f, 0x4e,
	0x44, 0x0c, 0x39, 0xae, 0x09, 0x9b, 0x72, 0xfb, 0x14, 0xad, 0xe7, 0x80, 0x53, 0xe0, 0x91, 0xaf,
	0xf6, 0xd8, 0x0b, 0x9e, 0xb9, 0x0f, 0xdf, 0x53, 0xf7, 0x4c, 0x69, 0xc2, 0x35, 0xad, 0xf5, 0x97,
	0x6c, 0x02, 0xf3, 0x47, 0xff, 0x60, 0x13, 0x8c, 0x7e, 0x58, 0x68, 0x55, 0x63, 0xf6, 0x18, 0x0d,
	0x04, 0xc9, 0x28, 0xa4, 0x91, 0x2e, 0x9a, 0xdd, 0x39, 0xee, 0xa2, 0x43, 0x5c, 0xdd, 0x1b, 0x97,
	0x8a, 0x66, 0xac, 0xfa, 0x62, 0x69, 0x66, 0x1f, 0x23, 0x7d, 0x1d, 0x95, 0x8b, 0x3a, 0xe3, 0x95,
	0x87, 0xce, 0x38, 0x1c, 0x18, 0x85, 0xb9, 0x25, 0x47, 0x68, 0xfb, 0x06, 0x38, 0x8b, 0x26, 0x94,
	0x7d, 0x28, 0x20, 0xcd, 0x20, 0xaa, 0x38, 0x63, 0x57, 0x6d, 0xd2, 0x4d, 0xed, 0x6d, 0x5b, 0xba,
	0x68, 0x2a, 0x27, 0xc7, 0xb7, 0xf7, 0x8e, 0x75, 0x77, 0xef, 0x58, 0xbf, 0xef, 0x1d, 0xeb, 0xf3,
	0xcc, 0xe9, 0xdc, 0xcd, 0x9c, 0xce, 0xcf, 0x99, 0xd3, 0x79, 0x7f, 0xa0, 0x1f, 0x0d, 0x5c, 0x55,
	0x9e, 0x6e, 0x54, 0xf3, 0x56, 0x1c, 0x05, 0xcf, 0x4d, 0x43, 0x4f, 0x00, 0x2a, 0xe0, 0xf1, 0xaa,
	0xea, 0xc7, 0x17, 0x7f, 0x02, 0x00, 0x00, 0xff, 0xff, 0xbf, 0x44, 0x80, 0x76, 0x66, 0x04, 0x00,
	0x00,
}

func (m *ClientState) Marshal() (dAtA []byte, err error) {
	size := m.Size()
	dAtA = make([]byte, size)
	n, err := m.MarshalToSizedBuffer(dAtA[:size])
	if err != nil {
		return nil, err
	}
	return dAtA[:n], nil
}

func (m *ClientState) MarshalTo(dAtA []byte) (int, error) {
	size := m.Size()
	return m.MarshalToSizedBuffer(dAtA[:size])
}

func (m *ClientState) MarshalToSizedBuffer(dAtA []byte) (int, error) {
	i := len(dAtA)
	_ = i
	var l int
	_ = l
	{
		size, err := m.LatestHeight.MarshalToSizedBuffer(dAtA[:i])
		if err != nil {
			return 0, err
		}
		i -= size
		i = encodeVarintCometbls(dAtA, i, uint64(size))
	}
	i--
	dAtA[i] = 0x32
	{
		size, err := m.FrozenHeight.MarshalToSizedBuffer(dAtA[:i])
		if err != nil {
			return 0, err
		}
		i -= size
		i = encodeVarintCometbls(dAtA, i, uint64(size))
	}
	i--
	dAtA[i] = 0x2a
	if m.MaxClockDrift != 0 {
		i = encodeVarintCometbls(dAtA, i, uint64(m.MaxClockDrift))
		i--
		dAtA[i] = 0x20
	}
	if m.UnbondingPeriod != 0 {
		i = encodeVarintCometbls(dAtA, i, uint64(m.UnbondingPeriod))
		i--
		dAtA[i] = 0x18
	}
	if m.TrustingPeriod != 0 {
		i = encodeVarintCometbls(dAtA, i, uint64(m.TrustingPeriod))
		i--
		dAtA[i] = 0x10
	}
	if len(m.ChainId) > 0 {
		i -= len(m.ChainId)
		copy(dAtA[i:], m.ChainId)
		i = encodeVarintCometbls(dAtA, i, uint64(len(m.ChainId)))
		i--
		dAtA[i] = 0xa
	}
	return len(dAtA) - i, nil
}

func (m *ConsensusState) Marshal() (dAtA []byte, err error) {
	size := m.Size()
	dAtA = make([]byte, size)
	n, err := m.MarshalToSizedBuffer(dAtA[:size])
	if err != nil {
		return nil, err
	}
	return dAtA[:n], nil
}

func (m *ConsensusState) MarshalTo(dAtA []byte) (int, error) {
	size := m.Size()
	return m.MarshalToSizedBuffer(dAtA[:size])
}

func (m *ConsensusState) MarshalToSizedBuffer(dAtA []byte) (int, error) {
	i := len(dAtA)
	_ = i
	var l int
	_ = l
	if len(m.NextValidatorsHash) > 0 {
		i -= len(m.NextValidatorsHash)
		copy(dAtA[i:], m.NextValidatorsHash)
		i = encodeVarintCometbls(dAtA, i, uint64(len(m.NextValidatorsHash)))
		i--
		dAtA[i] = 0x1a
	}
	{
		size, err := m.Root.MarshalToSizedBuffer(dAtA[:i])
		if err != nil {
			return 0, err
		}
		i -= size
		i = encodeVarintCometbls(dAtA, i, uint64(size))
	}
	i--
	dAtA[i] = 0x12
	if m.Timestamp != 0 {
		i = encodeVarintCometbls(dAtA, i, uint64(m.Timestamp))
		i--
		dAtA[i] = 0x8
	}
	return len(dAtA) - i, nil
}

func (m *Misbehaviour) Marshal() (dAtA []byte, err error) {
	size := m.Size()
	dAtA = make([]byte, size)
	n, err := m.MarshalToSizedBuffer(dAtA[:size])
	if err != nil {
		return nil, err
	}
	return dAtA[:n], nil
}

func (m *Misbehaviour) MarshalTo(dAtA []byte) (int, error) {
	size := m.Size()
	return m.MarshalToSizedBuffer(dAtA[:size])
}

func (m *Misbehaviour) MarshalToSizedBuffer(dAtA []byte) (int, error) {
	i := len(dAtA)
	_ = i
	var l int
	_ = l
	if m.Header_2 != nil {
		{
			size, err := m.Header_2.MarshalToSizedBuffer(dAtA[:i])
			if err != nil {
				return 0, err
			}
			i -= size
			i = encodeVarintCometbls(dAtA, i, uint64(size))
		}
		i--
		dAtA[i] = 0x12
	}
	if m.Header_1 != nil {
		{
			size, err := m.Header_1.MarshalToSizedBuffer(dAtA[:i])
			if err != nil {
				return 0, err
			}
			i -= size
			i = encodeVarintCometbls(dAtA, i, uint64(size))
		}
		i--
		dAtA[i] = 0xa
	}
	return len(dAtA) - i, nil
}

func (m *Header) Marshal() (dAtA []byte, err error) {
	size := m.Size()
	dAtA = make([]byte, size)
	n, err := m.MarshalToSizedBuffer(dAtA[:size])
	if err != nil {
		return nil, err
	}
	return dAtA[:n], nil
}

func (m *Header) MarshalTo(dAtA []byte) (int, error) {
	size := m.Size()
	return m.MarshalToSizedBuffer(dAtA[:size])
}

func (m *Header) MarshalToSizedBuffer(dAtA []byte) (int, error) {
	i := len(dAtA)
	_ = i
	var l int
	_ = l
	if len(m.ZeroKnowledgeProof) > 0 {
		i -= len(m.ZeroKnowledgeProof)
		copy(dAtA[i:], m.ZeroKnowledgeProof)
		i = encodeVarintCometbls(dAtA, i, uint64(len(m.ZeroKnowledgeProof)))
		i--
		dAtA[i] = 0x1a
	}
	if m.TrustedHeight != nil {
		{
			size, err := m.TrustedHeight.MarshalToSizedBuffer(dAtA[:i])
			if err != nil {
				return 0, err
			}
			i -= size
			i = encodeVarintCometbls(dAtA, i, uint64(size))
		}
		i--
		dAtA[i] = 0x12
	}
	if m.SignedHeader != nil {
		{
			size, err := m.SignedHeader.MarshalToSizedBuffer(dAtA[:i])
			if err != nil {
				return 0, err
			}
			i -= size
			i = encodeVarintCometbls(dAtA, i, uint64(size))
		}
		i--
		dAtA[i] = 0xa
	}
	return len(dAtA) - i, nil
}

func encodeVarintCometbls(dAtA []byte, offset int, v uint64) int {
	offset -= sovCometbls(v)
	base := offset
	for v >= 1<<7 {
		dAtA[offset] = uint8(v&0x7f | 0x80)
		v >>= 7
		offset++
	}
	dAtA[offset] = uint8(v)
	return base
}
func (m *ClientState) Size() (n int) {
	if m == nil {
		return 0
	}
	var l int
	_ = l
	l = len(m.ChainId)
	if l > 0 {
		n += 1 + l + sovCometbls(uint64(l))
	}
	if m.TrustingPeriod != 0 {
		n += 1 + sovCometbls(uint64(m.TrustingPeriod))
	}
	if m.UnbondingPeriod != 0 {
		n += 1 + sovCometbls(uint64(m.UnbondingPeriod))
	}
	if m.MaxClockDrift != 0 {
		n += 1 + sovCometbls(uint64(m.MaxClockDrift))
	}
	l = m.FrozenHeight.Size()
	n += 1 + l + sovCometbls(uint64(l))
	l = m.LatestHeight.Size()
	n += 1 + l + sovCometbls(uint64(l))
	return n
}

func (m *ConsensusState) Size() (n int) {
	if m == nil {
		return 0
	}
	var l int
	_ = l
	if m.Timestamp != 0 {
		n += 1 + sovCometbls(uint64(m.Timestamp))
	}
	l = m.Root.Size()
	n += 1 + l + sovCometbls(uint64(l))
	l = len(m.NextValidatorsHash)
	if l > 0 {
		n += 1 + l + sovCometbls(uint64(l))
	}
	return n
}

func (m *Misbehaviour) Size() (n int) {
	if m == nil {
		return 0
	}
	var l int
	_ = l
	if m.Header_1 != nil {
		l = m.Header_1.Size()
		n += 1 + l + sovCometbls(uint64(l))
	}
	if m.Header_2 != nil {
		l = m.Header_2.Size()
		n += 1 + l + sovCometbls(uint64(l))
	}
	return n
}

func (m *Header) Size() (n int) {
	if m == nil {
		return 0
	}
	var l int
	_ = l
	if m.SignedHeader != nil {
		l = m.SignedHeader.Size()
		n += 1 + l + sovCometbls(uint64(l))
	}
	if m.TrustedHeight != nil {
		l = m.TrustedHeight.Size()
		n += 1 + l + sovCometbls(uint64(l))
	}
	l = len(m.ZeroKnowledgeProof)
	if l > 0 {
		n += 1 + l + sovCometbls(uint64(l))
	}
	return n
}

func sovCometbls(x uint64) (n int) {
	return (math_bits.Len64(x|1) + 6) / 7
}
func sozCometbls(x uint64) (n int) {
	return sovCometbls(uint64((x << 1) ^ uint64((int64(x) >> 63))))
}
func (m *ClientState) Unmarshal(dAtA []byte) error {
	l := len(dAtA)
	iNdEx := 0
	for iNdEx < l {
		preIndex := iNdEx
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return ErrIntOverflowCometbls
			}
			if iNdEx >= l {
				return io.ErrUnexpectedEOF
			}
			b := dAtA[iNdEx]
			iNdEx++
			wire |= uint64(b&0x7F) << shift
			if b < 0x80 {
				break
			}
		}
		fieldNum := int32(wire >> 3)
		wireType := int(wire & 0x7)
		if wireType == 4 {
			return fmt.Errorf("proto: ClientState: wiretype end group for non-group")
		}
		if fieldNum <= 0 {
			return fmt.Errorf("proto: ClientState: illegal tag %d (wire type %d)", fieldNum, wire)
		}
		switch fieldNum {
		case 1:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field ChainId", wireType)
			}
			var stringLen uint64
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				stringLen |= uint64(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			intStringLen := int(stringLen)
			if intStringLen < 0 {
				return ErrInvalidLengthCometbls
			}
			postIndex := iNdEx + intStringLen
			if postIndex < 0 {
				return ErrInvalidLengthCometbls
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			m.ChainId = string(dAtA[iNdEx:postIndex])
			iNdEx = postIndex
		case 2:
			if wireType != 0 {
				return fmt.Errorf("proto: wrong wireType = %d for field TrustingPeriod", wireType)
			}
			m.TrustingPeriod = 0
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				m.TrustingPeriod |= uint64(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
		case 3:
			if wireType != 0 {
				return fmt.Errorf("proto: wrong wireType = %d for field UnbondingPeriod", wireType)
			}
			m.UnbondingPeriod = 0
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				m.UnbondingPeriod |= uint64(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
		case 4:
			if wireType != 0 {
				return fmt.Errorf("proto: wrong wireType = %d for field MaxClockDrift", wireType)
			}
			m.MaxClockDrift = 0
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				m.MaxClockDrift |= uint64(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
		case 5:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field FrozenHeight", wireType)
			}
			var msglen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				msglen |= int(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if msglen < 0 {
				return ErrInvalidLengthCometbls
			}
			postIndex := iNdEx + msglen
			if postIndex < 0 {
				return ErrInvalidLengthCometbls
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			if err := m.FrozenHeight.Unmarshal(dAtA[iNdEx:postIndex]); err != nil {
				return err
			}
			iNdEx = postIndex
		case 6:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field LatestHeight", wireType)
			}
			var msglen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				msglen |= int(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if msglen < 0 {
				return ErrInvalidLengthCometbls
			}
			postIndex := iNdEx + msglen
			if postIndex < 0 {
				return ErrInvalidLengthCometbls
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			if err := m.LatestHeight.Unmarshal(dAtA[iNdEx:postIndex]); err != nil {
				return err
			}
			iNdEx = postIndex
		default:
			iNdEx = preIndex
			skippy, err := skipCometbls(dAtA[iNdEx:])
			if err != nil {
				return err
			}
			if (skippy < 0) || (iNdEx+skippy) < 0 {
				return ErrInvalidLengthCometbls
			}
			if (iNdEx + skippy) > l {
				return io.ErrUnexpectedEOF
			}
			iNdEx += skippy
		}
	}

	if iNdEx > l {
		return io.ErrUnexpectedEOF
	}
	return nil
}
func (m *ConsensusState) Unmarshal(dAtA []byte) error {
	l := len(dAtA)
	iNdEx := 0
	for iNdEx < l {
		preIndex := iNdEx
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return ErrIntOverflowCometbls
			}
			if iNdEx >= l {
				return io.ErrUnexpectedEOF
			}
			b := dAtA[iNdEx]
			iNdEx++
			wire |= uint64(b&0x7F) << shift
			if b < 0x80 {
				break
			}
		}
		fieldNum := int32(wire >> 3)
		wireType := int(wire & 0x7)
		if wireType == 4 {
			return fmt.Errorf("proto: ConsensusState: wiretype end group for non-group")
		}
		if fieldNum <= 0 {
			return fmt.Errorf("proto: ConsensusState: illegal tag %d (wire type %d)", fieldNum, wire)
		}
		switch fieldNum {
		case 1:
			if wireType != 0 {
				return fmt.Errorf("proto: wrong wireType = %d for field Timestamp", wireType)
			}
			m.Timestamp = 0
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				m.Timestamp |= uint64(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
		case 2:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field Root", wireType)
			}
			var msglen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				msglen |= int(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if msglen < 0 {
				return ErrInvalidLengthCometbls
			}
			postIndex := iNdEx + msglen
			if postIndex < 0 {
				return ErrInvalidLengthCometbls
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			if err := m.Root.Unmarshal(dAtA[iNdEx:postIndex]); err != nil {
				return err
			}
			iNdEx = postIndex
		case 3:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field NextValidatorsHash", wireType)
			}
			var byteLen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				byteLen |= int(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if byteLen < 0 {
				return ErrInvalidLengthCometbls
			}
			postIndex := iNdEx + byteLen
			if postIndex < 0 {
				return ErrInvalidLengthCometbls
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			m.NextValidatorsHash = append(m.NextValidatorsHash[:0], dAtA[iNdEx:postIndex]...)
			if m.NextValidatorsHash == nil {
				m.NextValidatorsHash = []byte{}
			}
			iNdEx = postIndex
		default:
			iNdEx = preIndex
			skippy, err := skipCometbls(dAtA[iNdEx:])
			if err != nil {
				return err
			}
			if (skippy < 0) || (iNdEx+skippy) < 0 {
				return ErrInvalidLengthCometbls
			}
			if (iNdEx + skippy) > l {
				return io.ErrUnexpectedEOF
			}
			iNdEx += skippy
		}
	}

	if iNdEx > l {
		return io.ErrUnexpectedEOF
	}
	return nil
}
func (m *Misbehaviour) Unmarshal(dAtA []byte) error {
	l := len(dAtA)
	iNdEx := 0
	for iNdEx < l {
		preIndex := iNdEx
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return ErrIntOverflowCometbls
			}
			if iNdEx >= l {
				return io.ErrUnexpectedEOF
			}
			b := dAtA[iNdEx]
			iNdEx++
			wire |= uint64(b&0x7F) << shift
			if b < 0x80 {
				break
			}
		}
		fieldNum := int32(wire >> 3)
		wireType := int(wire & 0x7)
		if wireType == 4 {
			return fmt.Errorf("proto: Misbehaviour: wiretype end group for non-group")
		}
		if fieldNum <= 0 {
			return fmt.Errorf("proto: Misbehaviour: illegal tag %d (wire type %d)", fieldNum, wire)
		}
		switch fieldNum {
		case 1:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field Header_1", wireType)
			}
			var msglen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				msglen |= int(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if msglen < 0 {
				return ErrInvalidLengthCometbls
			}
			postIndex := iNdEx + msglen
			if postIndex < 0 {
				return ErrInvalidLengthCometbls
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			if m.Header_1 == nil {
				m.Header_1 = &Header{}
			}
			if err := m.Header_1.Unmarshal(dAtA[iNdEx:postIndex]); err != nil {
				return err
			}
			iNdEx = postIndex
		case 2:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field Header_2", wireType)
			}
			var msglen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				msglen |= int(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if msglen < 0 {
				return ErrInvalidLengthCometbls
			}
			postIndex := iNdEx + msglen
			if postIndex < 0 {
				return ErrInvalidLengthCometbls
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			if m.Header_2 == nil {
				m.Header_2 = &Header{}
			}
			if err := m.Header_2.Unmarshal(dAtA[iNdEx:postIndex]); err != nil {
				return err
			}
			iNdEx = postIndex
		default:
			iNdEx = preIndex
			skippy, err := skipCometbls(dAtA[iNdEx:])
			if err != nil {
				return err
			}
			if (skippy < 0) || (iNdEx+skippy) < 0 {
				return ErrInvalidLengthCometbls
			}
			if (iNdEx + skippy) > l {
				return io.ErrUnexpectedEOF
			}
			iNdEx += skippy
		}
	}

	if iNdEx > l {
		return io.ErrUnexpectedEOF
	}
	return nil
}
func (m *Header) Unmarshal(dAtA []byte) error {
	l := len(dAtA)
	iNdEx := 0
	for iNdEx < l {
		preIndex := iNdEx
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return ErrIntOverflowCometbls
			}
			if iNdEx >= l {
				return io.ErrUnexpectedEOF
			}
			b := dAtA[iNdEx]
			iNdEx++
			wire |= uint64(b&0x7F) << shift
			if b < 0x80 {
				break
			}
		}
		fieldNum := int32(wire >> 3)
		wireType := int(wire & 0x7)
		if wireType == 4 {
			return fmt.Errorf("proto: Header: wiretype end group for non-group")
		}
		if fieldNum <= 0 {
			return fmt.Errorf("proto: Header: illegal tag %d (wire type %d)", fieldNum, wire)
		}
		switch fieldNum {
		case 1:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field SignedHeader", wireType)
			}
			var msglen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				msglen |= int(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if msglen < 0 {
				return ErrInvalidLengthCometbls
			}
			postIndex := iNdEx + msglen
			if postIndex < 0 {
				return ErrInvalidLengthCometbls
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			if m.SignedHeader == nil {
				m.SignedHeader = &types2.SignedHeader{}
			}
			if err := m.SignedHeader.Unmarshal(dAtA[iNdEx:postIndex]); err != nil {
				return err
			}
			iNdEx = postIndex
		case 2:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field TrustedHeight", wireType)
			}
			var msglen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				msglen |= int(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if msglen < 0 {
				return ErrInvalidLengthCometbls
			}
			postIndex := iNdEx + msglen
			if postIndex < 0 {
				return ErrInvalidLengthCometbls
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			if m.TrustedHeight == nil {
				m.TrustedHeight = &types.Height{}
			}
			if err := m.TrustedHeight.Unmarshal(dAtA[iNdEx:postIndex]); err != nil {
				return err
			}
			iNdEx = postIndex
		case 3:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field ZeroKnowledgeProof", wireType)
			}
			var byteLen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				byteLen |= int(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if byteLen < 0 {
				return ErrInvalidLengthCometbls
			}
			postIndex := iNdEx + byteLen
			if postIndex < 0 {
				return ErrInvalidLengthCometbls
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			m.ZeroKnowledgeProof = append(m.ZeroKnowledgeProof[:0], dAtA[iNdEx:postIndex]...)
			if m.ZeroKnowledgeProof == nil {
				m.ZeroKnowledgeProof = []byte{}
			}
			iNdEx = postIndex
		default:
			iNdEx = preIndex
			skippy, err := skipCometbls(dAtA[iNdEx:])
			if err != nil {
				return err
			}
			if (skippy < 0) || (iNdEx+skippy) < 0 {
				return ErrInvalidLengthCometbls
			}
			if (iNdEx + skippy) > l {
				return io.ErrUnexpectedEOF
			}
			iNdEx += skippy
		}
	}

	if iNdEx > l {
		return io.ErrUnexpectedEOF
	}
	return nil
}
func skipCometbls(dAtA []byte) (n int, err error) {
	l := len(dAtA)
	iNdEx := 0
	depth := 0
	for iNdEx < l {
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return 0, ErrIntOverflowCometbls
			}
			if iNdEx >= l {
				return 0, io.ErrUnexpectedEOF
			}
			b := dAtA[iNdEx]
			iNdEx++
			wire |= (uint64(b) & 0x7F) << shift
			if b < 0x80 {
				break
			}
		}
		wireType := int(wire & 0x7)
		switch wireType {
		case 0:
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return 0, ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return 0, io.ErrUnexpectedEOF
				}
				iNdEx++
				if dAtA[iNdEx-1] < 0x80 {
					break
				}
			}
		case 1:
			iNdEx += 8
		case 2:
			var length int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return 0, ErrIntOverflowCometbls
				}
				if iNdEx >= l {
					return 0, io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				length |= (int(b) & 0x7F) << shift
				if b < 0x80 {
					break
				}
			}
			if length < 0 {
				return 0, ErrInvalidLengthCometbls
			}
			iNdEx += length
		case 3:
			depth++
		case 4:
			if depth == 0 {
				return 0, ErrUnexpectedEndOfGroupCometbls
			}
			depth--
		case 5:
			iNdEx += 4
		default:
			return 0, fmt.Errorf("proto: illegal wireType %d", wireType)
		}
		if iNdEx < 0 {
			return 0, ErrInvalidLengthCometbls
		}
		if depth == 0 {
			return iNdEx, nil
		}
	}
	return 0, io.ErrUnexpectedEOF
}

var (
	ErrInvalidLengthCometbls        = fmt.Errorf("proto: negative length found during unmarshaling")
	ErrIntOverflowCometbls          = fmt.Errorf("proto: integer overflow")
	ErrUnexpectedEndOfGroupCometbls = fmt.Errorf("proto: unexpected end of group")
)
