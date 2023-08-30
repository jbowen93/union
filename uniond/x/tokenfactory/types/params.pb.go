//lint:file-ignore SA1019 This code is generated
// Code generated by protoc-gen-gogo. DO NOT EDIT.
// source: tokenfactory/v1beta1/params.proto

package types

import (
	fmt "fmt"
	github_com_cosmos_cosmos_sdk_types "github.com/cosmos/cosmos-sdk/types"
	types "github.com/cosmos/cosmos-sdk/types"
	_ "github.com/cosmos/gogoproto/gogoproto"
	proto "github.com/cosmos/gogoproto/proto"
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

// Params defines the parameters for the tokenfactory module.
type Params struct {
	DenomCreationFee github_com_cosmos_cosmos_sdk_types.Coins `protobuf:"bytes,1,rep,name=denom_creation_fee,json=denomCreationFee,proto3,castrepeated=github.com/cosmos/cosmos-sdk/types.Coins" json:"denom_creation_fee" yaml:"denom_creation_fee"`
	// if denom_creation_fee is an empty array, then this field is used to add more gas consumption
	// to the base cost.
	// https://github.com/CosmWasm/token-factory/issues/11
	DenomCreationGasConsume uint64 `protobuf:"varint,2,opt,name=denom_creation_gas_consume,json=denomCreationGasConsume,proto3" json:"denom_creation_gas_consume,omitempty" yaml:"denom_creation_gas_consume"`
}

func (m *Params) Reset()         { *m = Params{} }
func (m *Params) String() string { return proto.CompactTextString(m) }
func (*Params) ProtoMessage()    {}
func (*Params) Descriptor() ([]byte, []int) {
	return fileDescriptor_4d491a2fda25be4d, []int{0}
}
func (m *Params) XXX_Unmarshal(b []byte) error {
	return m.Unmarshal(b)
}
func (m *Params) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	if deterministic {
		return xxx_messageInfo_Params.Marshal(b, m, deterministic)
	} else {
		b = b[:cap(b)]
		n, err := m.MarshalToSizedBuffer(b)
		if err != nil {
			return nil, err
		}
		return b[:n], nil
	}
}
func (m *Params) XXX_Merge(src proto.Message) {
	xxx_messageInfo_Params.Merge(m, src)
}
func (m *Params) XXX_Size() int {
	return m.Size()
}
func (m *Params) XXX_DiscardUnknown() {
	xxx_messageInfo_Params.DiscardUnknown(m)
}

var xxx_messageInfo_Params proto.InternalMessageInfo

func (m *Params) GetDenomCreationFee() github_com_cosmos_cosmos_sdk_types.Coins {
	if m != nil {
		return m.DenomCreationFee
	}
	return nil
}

func (m *Params) GetDenomCreationGasConsume() uint64 {
	if m != nil {
		return m.DenomCreationGasConsume
	}
	return 0
}

func init() {
	proto.RegisterType((*Params)(nil), "tokenfactory.v1beta1.Params")
}

func init() { proto.RegisterFile("tokenfactory/v1beta1/params.proto", fileDescriptor_4d491a2fda25be4d) }

var fileDescriptor_4d491a2fda25be4d = []byte{
	// 312 bytes of a gzipped FileDescriptorProto
	0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0xff, 0x6c, 0x91, 0x31, 0x4e, 0xf3, 0x30,
	0x18, 0x86, 0xe3, 0xfe, 0xbf, 0x3a, 0x84, 0x05, 0x45, 0x95, 0x68, 0x33, 0x38, 0x6d, 0xa6, 0x32,
	0x60, 0xab, 0xc0, 0xc4, 0xd8, 0x4a, 0x30, 0x21, 0xa1, 0x8e, 0x2c, 0x95, 0xe3, 0xba, 0x21, 0x2a,
	0xf6, 0x57, 0xc5, 0x2e, 0xa2, 0x47, 0x60, 0x63, 0xe2, 0x10, 0x9c, 0x24, 0x63, 0x47, 0xa6, 0x80,
	0x92, 0x1b, 0xf4, 0x04, 0xa8, 0x4e, 0x40, 0x2d, 0x30, 0xd9, 0xd2, 0xf7, 0xf8, 0x79, 0x5f, 0xf9,
	0x73, 0x7b, 0x06, 0xe6, 0x42, 0xcd, 0x18, 0x37, 0x90, 0xae, 0xe8, 0xc3, 0x20, 0x12, 0x86, 0x0d,
	0xe8, 0x82, 0xa5, 0x4c, 0x6a, 0xb2, 0x48, 0xc1, 0x80, 0xd7, 0xda, 0x45, 0x48, 0x8d, 0xf8, 0xad,
	0x18, 0x62, 0xb0, 0x00, 0xdd, 0xde, 0x2a, 0xd6, 0xc7, 0x1c, 0xb4, 0x04, 0x4d, 0x23, 0xa6, 0xc5,
	0xb7, 0x8d, 0x43, 0xa2, 0xaa, 0x79, 0xf8, 0xd4, 0x70, 0x9b, 0x37, 0x56, 0xee, 0xbd, 0x20, 0xd7,
	0x9b, 0x0a, 0x05, 0x72, 0xc2, 0x53, 0xc1, 0x4c, 0x02, 0x6a, 0x32, 0x13, 0xa2, 0x8d, 0xba, 0xff,
	0xfa, 0x07, 0xa7, 0x1d, 0x52, 0x89, 0xc8, 0x56, 0xf4, 0x95, 0x49, 0x46, 0x90, 0xa8, 0xe1, 0x75,
	0x96, 0x07, 0xce, 0x26, 0x0f, 0x3a, 0x2b, 0x26, 0xef, 0x2f, 0xc2, 0xdf, 0x8a, 0xf0, 0xf5, 0x3d,
	0xe8, 0xc7, 0x89, 0xb9, 0x5b, 0x46, 0x84, 0x83, 0xa4, 0x75, 0xa5, 0xea, 0x38, 0xd1, 0xd3, 0x39,
	0x35, 0xab, 0x85, 0xd0, 0xd6, 0xa6, 0xc7, 0x87, 0x56, 0x30, 0xaa, 0xdf, 0x5f, 0x0a, 0xe1, 0xcd,
	0x5c, 0xff, 0x87, 0x34, 0x66, 0x7a, 0xc2, 0x41, 0xe9, 0xa5, 0x14, 0xed, 0x46, 0x17, 0xf5, 0xff,
	0x0f, 0x8f, 0xb3, 0x3c, 0x40, 0x9b, 0x3c, 0xe8, 0xfd, 0x59, 0x62, 0x87, 0x0f, 0xc7, 0x47, 0x7b,
	0x01, 0x57, 0x4c, 0x8f, 0xaa, 0xc9, 0xf0, 0x3c, 0x2b, 0x30, 0x5a, 0x17, 0x18, 0x7d, 0x14, 0x18,
	0x3d, 0x97, 0xd8, 0x59, 0x97, 0xd8, 0x79, 0x2b, 0xb1, 0x73, 0xeb, 0x2f, 0x55, 0x02, 0x8a, 0x3e,
	0xd2, 0xbd, 0xe5, 0xd8, 0xd6, 0x51, 0xd3, 0x7e, 0xe4, 0xd9, 0x67, 0x00, 0x00, 0x00, 0xff, 0xff,
	0x1a, 0xd9, 0x85, 0xa2, 0xb9, 0x01, 0x00, 0x00,
}

func (m *Params) Marshal() (dAtA []byte, err error) {
	size := m.Size()
	dAtA = make([]byte, size)
	n, err := m.MarshalToSizedBuffer(dAtA[:size])
	if err != nil {
		return nil, err
	}
	return dAtA[:n], nil
}

func (m *Params) MarshalTo(dAtA []byte) (int, error) {
	size := m.Size()
	return m.MarshalToSizedBuffer(dAtA[:size])
}

func (m *Params) MarshalToSizedBuffer(dAtA []byte) (int, error) {
	i := len(dAtA)
	_ = i
	var l int
	_ = l
	if m.DenomCreationGasConsume != 0 {
		i = encodeVarintParams(dAtA, i, uint64(m.DenomCreationGasConsume))
		i--
		dAtA[i] = 0x10
	}
	if len(m.DenomCreationFee) > 0 {
		for iNdEx := len(m.DenomCreationFee) - 1; iNdEx >= 0; iNdEx-- {
			{
				size, err := m.DenomCreationFee[iNdEx].MarshalToSizedBuffer(dAtA[:i])
				if err != nil {
					return 0, err
				}
				i -= size
				i = encodeVarintParams(dAtA, i, uint64(size))
			}
			i--
			dAtA[i] = 0xa
		}
	}
	return len(dAtA) - i, nil
}

func encodeVarintParams(dAtA []byte, offset int, v uint64) int {
	offset -= sovParams(v)
	base := offset
	for v >= 1<<7 {
		dAtA[offset] = uint8(v&0x7f | 0x80)
		v >>= 7
		offset++
	}
	dAtA[offset] = uint8(v)
	return base
}
func (m *Params) Size() (n int) {
	if m == nil {
		return 0
	}
	var l int
	_ = l
	if len(m.DenomCreationFee) > 0 {
		for _, e := range m.DenomCreationFee {
			l = e.Size()
			n += 1 + l + sovParams(uint64(l))
		}
	}
	if m.DenomCreationGasConsume != 0 {
		n += 1 + sovParams(uint64(m.DenomCreationGasConsume))
	}
	return n
}

func sovParams(x uint64) (n int) {
	return (math_bits.Len64(x|1) + 6) / 7
}
func sozParams(x uint64) (n int) {
	return sovParams(uint64((x << 1) ^ uint64((int64(x) >> 63))))
}
func (m *Params) Unmarshal(dAtA []byte) error {
	l := len(dAtA)
	iNdEx := 0
	for iNdEx < l {
		preIndex := iNdEx
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return ErrIntOverflowParams
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
			return fmt.Errorf("proto: Params: wiretype end group for non-group")
		}
		if fieldNum <= 0 {
			return fmt.Errorf("proto: Params: illegal tag %d (wire type %d)", fieldNum, wire)
		}
		switch fieldNum {
		case 1:
			if wireType != 2 {
				return fmt.Errorf("proto: wrong wireType = %d for field DenomCreationFee", wireType)
			}
			var msglen int
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowParams
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
				return ErrInvalidLengthParams
			}
			postIndex := iNdEx + msglen
			if postIndex < 0 {
				return ErrInvalidLengthParams
			}
			if postIndex > l {
				return io.ErrUnexpectedEOF
			}
			m.DenomCreationFee = append(m.DenomCreationFee, types.Coin{})
			if err := m.DenomCreationFee[len(m.DenomCreationFee)-1].Unmarshal(dAtA[iNdEx:postIndex]); err != nil {
				return err
			}
			iNdEx = postIndex
		case 2:
			if wireType != 0 {
				return fmt.Errorf("proto: wrong wireType = %d for field DenomCreationGasConsume", wireType)
			}
			m.DenomCreationGasConsume = 0
			for shift := uint(0); ; shift += 7 {
				if shift >= 64 {
					return ErrIntOverflowParams
				}
				if iNdEx >= l {
					return io.ErrUnexpectedEOF
				}
				b := dAtA[iNdEx]
				iNdEx++
				m.DenomCreationGasConsume |= uint64(b&0x7F) << shift
				if b < 0x80 {
					break
				}
			}
		default:
			iNdEx = preIndex
			skippy, err := skipParams(dAtA[iNdEx:])
			if err != nil {
				return err
			}
			if (skippy < 0) || (iNdEx+skippy) < 0 {
				return ErrInvalidLengthParams
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
func skipParams(dAtA []byte) (n int, err error) {
	l := len(dAtA)
	iNdEx := 0
	depth := 0
	for iNdEx < l {
		var wire uint64
		for shift := uint(0); ; shift += 7 {
			if shift >= 64 {
				return 0, ErrIntOverflowParams
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
					return 0, ErrIntOverflowParams
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
					return 0, ErrIntOverflowParams
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
				return 0, ErrInvalidLengthParams
			}
			iNdEx += length
		case 3:
			depth++
		case 4:
			if depth == 0 {
				return 0, ErrUnexpectedEndOfGroupParams
			}
			depth--
		case 5:
			iNdEx += 4
		default:
			return 0, fmt.Errorf("proto: illegal wireType %d", wireType)
		}
		if iNdEx < 0 {
			return 0, ErrInvalidLengthParams
		}
		if depth == 0 {
			return iNdEx, nil
		}
	}
	return 0, io.ErrUnexpectedEOF
}

var (
	ErrInvalidLengthParams        = fmt.Errorf("proto: negative length found during unmarshaling")
	ErrIntOverflowParams          = fmt.Errorf("proto: integer overflow")
	ErrUnexpectedEndOfGroupParams = fmt.Errorf("proto: unexpected end of group")
)
