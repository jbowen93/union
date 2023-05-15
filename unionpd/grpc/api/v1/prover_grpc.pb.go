// Code generated by protoc-gen-go-grpc. DO NOT EDIT.
// versions:
// - protoc-gen-go-grpc v1.3.0
// - protoc             v3.21.12
// source: api/v1/prover.proto

package grpc

import (
	context "context"
	grpc "google.golang.org/grpc"
	codes "google.golang.org/grpc/codes"
	status "google.golang.org/grpc/status"
)

// This is a compile-time assertion to ensure that this generated file
// is compatible with the grpc package it is being compiled against.
// Requires gRPC-Go v1.32.0 or later.
const _ = grpc.SupportPackageIsVersion7

const (
	UnionProverAPI_Prove_FullMethodName            = "/union.prover.api.v1.UnionProverAPI/Prove"
	UnionProverAPI_Verify_FullMethodName           = "/union.prover.api.v1.UnionProverAPI/Verify"
	UnionProverAPI_GenerateContract_FullMethodName = "/union.prover.api.v1.UnionProverAPI/GenerateContract"
)

// UnionProverAPIClient is the client API for UnionProverAPI service.
//
// For semantics around ctx use and closing/ending streaming RPCs, please refer to https://pkg.go.dev/google.golang.org/grpc/?tab=doc#ClientConn.NewStream.
type UnionProverAPIClient interface {
	Prove(ctx context.Context, in *ProveRequest, opts ...grpc.CallOption) (*ProveResponse, error)
	Verify(ctx context.Context, in *VerifyRequest, opts ...grpc.CallOption) (*VerifyResponse, error)
	GenerateContract(ctx context.Context, in *GenerateContractRequest, opts ...grpc.CallOption) (*GenerateContractResponse, error)
}

type unionProverAPIClient struct {
	cc grpc.ClientConnInterface
}

func NewUnionProverAPIClient(cc grpc.ClientConnInterface) UnionProverAPIClient {
	return &unionProverAPIClient{cc}
}

func (c *unionProverAPIClient) Prove(ctx context.Context, in *ProveRequest, opts ...grpc.CallOption) (*ProveResponse, error) {
	out := new(ProveResponse)
	err := c.cc.Invoke(ctx, UnionProverAPI_Prove_FullMethodName, in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *unionProverAPIClient) Verify(ctx context.Context, in *VerifyRequest, opts ...grpc.CallOption) (*VerifyResponse, error) {
	out := new(VerifyResponse)
	err := c.cc.Invoke(ctx, UnionProverAPI_Verify_FullMethodName, in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *unionProverAPIClient) GenerateContract(ctx context.Context, in *GenerateContractRequest, opts ...grpc.CallOption) (*GenerateContractResponse, error) {
	out := new(GenerateContractResponse)
	err := c.cc.Invoke(ctx, UnionProverAPI_GenerateContract_FullMethodName, in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

// UnionProverAPIServer is the server API for UnionProverAPI service.
// All implementations must embed UnimplementedUnionProverAPIServer
// for forward compatibility
type UnionProverAPIServer interface {
	Prove(context.Context, *ProveRequest) (*ProveResponse, error)
	Verify(context.Context, *VerifyRequest) (*VerifyResponse, error)
	GenerateContract(context.Context, *GenerateContractRequest) (*GenerateContractResponse, error)
	mustEmbedUnimplementedUnionProverAPIServer()
}

// UnimplementedUnionProverAPIServer must be embedded to have forward compatible implementations.
type UnimplementedUnionProverAPIServer struct {
}

func (UnimplementedUnionProverAPIServer) Prove(context.Context, *ProveRequest) (*ProveResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method Prove not implemented")
}
func (UnimplementedUnionProverAPIServer) Verify(context.Context, *VerifyRequest) (*VerifyResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method Verify not implemented")
}
func (UnimplementedUnionProverAPIServer) GenerateContract(context.Context, *GenerateContractRequest) (*GenerateContractResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method GenerateContract not implemented")
}
func (UnimplementedUnionProverAPIServer) mustEmbedUnimplementedUnionProverAPIServer() {}

// UnsafeUnionProverAPIServer may be embedded to opt out of forward compatibility for this service.
// Use of this interface is not recommended, as added methods to UnionProverAPIServer will
// result in compilation errors.
type UnsafeUnionProverAPIServer interface {
	mustEmbedUnimplementedUnionProverAPIServer()
}

func RegisterUnionProverAPIServer(s grpc.ServiceRegistrar, srv UnionProverAPIServer) {
	s.RegisterService(&UnionProverAPI_ServiceDesc, srv)
}

func _UnionProverAPI_Prove_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(ProveRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(UnionProverAPIServer).Prove(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: UnionProverAPI_Prove_FullMethodName,
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(UnionProverAPIServer).Prove(ctx, req.(*ProveRequest))
	}
	return interceptor(ctx, in, info, handler)
}

func _UnionProverAPI_Verify_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(VerifyRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(UnionProverAPIServer).Verify(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: UnionProverAPI_Verify_FullMethodName,
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(UnionProverAPIServer).Verify(ctx, req.(*VerifyRequest))
	}
	return interceptor(ctx, in, info, handler)
}

func _UnionProverAPI_GenerateContract_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(GenerateContractRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(UnionProverAPIServer).GenerateContract(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: UnionProverAPI_GenerateContract_FullMethodName,
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(UnionProverAPIServer).GenerateContract(ctx, req.(*GenerateContractRequest))
	}
	return interceptor(ctx, in, info, handler)
}

// UnionProverAPI_ServiceDesc is the grpc.ServiceDesc for UnionProverAPI service.
// It's only intended for direct use with grpc.RegisterService,
// and not to be introspected or modified (even as a copy)
var UnionProverAPI_ServiceDesc = grpc.ServiceDesc{
	ServiceName: "union.prover.api.v1.UnionProverAPI",
	HandlerType: (*UnionProverAPIServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "Prove",
			Handler:    _UnionProverAPI_Prove_Handler,
		},
		{
			MethodName: "Verify",
			Handler:    _UnionProverAPI_Verify_Handler,
		},
		{
			MethodName: "GenerateContract",
			Handler:    _UnionProverAPI_GenerateContract_Handler,
		},
	},
	Streams:  []grpc.StreamDesc{},
	Metadata: "api/v1/prover.proto",
}
