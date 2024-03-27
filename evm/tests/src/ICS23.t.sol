pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import "../../contracts/proto/cosmos/ics23/v1/proofs.sol";
import "../../contracts/proto/ibc/core/commitment/v1/commitment.sol";
import "../../contracts/lib/ICS23.sol";
import "../../contracts/lib/CometblsHelp.sol";
import "../../contracts/clients/ICS23MembershipVerifier.sol";

contract ICS23Test is Test {
    function test_decode() public {
        bytes memory proof =
            hex"0acf0112cc010a0441413d3d1ac3010a20636c69656e74732f30392d6c6f63616c686f73742f636c69656e74537461746512350a2a2f6962632e6c69676874636c69656e74732e6c6f63616c686f73742e76322e436c69656e74537461746512070a05080210d9021a0c0801180120012a040002b205222c080112050204b205201a212075c4910f51207d3c65960120fe931f138e2624668d75869f51b8442593dd5eab222c08011205060ab205201a2120c617eef82d350859fc4c1b0118079fad20ecd1e5fe2294c86a86e94c0d14ca550afe010afb010a03696263122016df45c79ca168d31360706db2979a792fcd9c2371478a77af951f39ec59487d1a090801180120012a0100222708011201011a20c505c0fd48b1cf2b65619f12b3144e19c60b4e6b62525ee936be93d041623ebf222708011201011a20dd55097e1841a5872eb9dbdf5ca65b54143f01aa24b598538786884a824191602225080112210182c1cabef1a0591f8016c06a396858762bb949a7dfc03e568b4273b3c205273d222508011221010d9399f8bc11620accd23dbfaeca52b7cda51a92ec71c044cd1d39a34bea2826222708011201011a2022ecbf124eff995ecf01998dd8346b71810af164e192feeb4d4287085128b9df";

        CosmosIcs23V1CommitmentProof.Data memory commitmentProof =
            CosmosIcs23V1CommitmentProof.decode(proof);
    }

    function getExists(IbcCoreCommitmentV1MerkleProof.Data memory decoded)
        internal
        returns (
            UnionIcs23.ExistenceProof memory,
            UnionIcs23.ExistenceProof memory
        )
    {
        UnionIcs23.ExistenceProof memory existProof;
        existProof.key = decoded.proofs[0].exist.key;
        existProof.value = decoded.proofs[0].exist.value;
        existProof.leafPrefix = decoded.proofs[0].exist.leaf.prefix;

        existProof.path =
            new UnionIcs23.InnerOp[](decoded.proofs[0].exist.path.length);
        for (uint256 i = 0; i < existProof.path.length; i++) {
            existProof.path[i].prefix = decoded.proofs[0].exist.path[i].prefix;
            existProof.path[i].suffix = decoded.proofs[0].exist.path[i].suffix;
        }

        UnionIcs23.ExistenceProof memory existProof2;
        existProof2.key = decoded.proofs[1].exist.key;
        existProof2.value = decoded.proofs[1].exist.value;
        existProof2.leafPrefix = decoded.proofs[1].exist.leaf.prefix;

        existProof2.path =
            new UnionIcs23.InnerOp[](decoded.proofs[1].exist.path.length);
        for (uint256 i = 0; i < existProof2.path.length; i++) {
            existProof2.path[i].prefix = decoded.proofs[1].exist.path[i].prefix;
            existProof2.path[i].suffix = decoded.proofs[1].exist.path[i].suffix;
        }

        return (existProof, existProof2);
    }

    function test_accountExist() public {
        vm.pauseGasMetering();
        bytes memory root =
            hex"B802C903BEFE08624832AAF853DBEA4EDE1F7D50E88BEDD6317831F45CC74A3D";
        bytes memory proof =
            hex"0afa020af7020a15014152090b0c95c948edc407995560feed4a9df81e129e010a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e31673966716a7a63766a687935336d77797137763432633837613439666d37713772646568386312460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103820c4b94dccd7d74706216c426fe884d9a4404410df69d6421899595c5a9c122180120011a0b0801180120012a0300020222290801122502040220170c890f01b9fa9ab803511bbc7be7c25359309f04d021a72e0a9b93b8ff72c020222b08011204040802201a2120a89a7b1aedf861a8c6316009af3d19448bfe8834dfb5546c7e1af7f95c3000b4222b08011204061002201a212029347d33c119e85fc1335f43ad17c4a1986ad44c71837158ceffd36e2f38f986222b080112040a3002201a2120e284b7ed0385d018b1ffcd6f33bf6ac575fb7731704d0ae71be278bd8bf5e0b50a80020afd010a03616363122082d7d632a58654a81bb6764379eff4b6e641e96620a12dac0e250e6caf94f7761a090801180120012a010022250801122101ba30cf8122e71a87fea08d0da9499e0373495a64e1648de8f08ca1a73e1fc1a8222708011201011a208a19e0585632ebada293099d24f28707d453266ae7ded6e854dfd8a025c7ce71222708011201011a204a22410f42f7706402b38c460e74d712c95cea8e6e370c691f43c0abf3f4e104222708011201011a20b999d9a62cbd36a843f207580c4802d194e6441f7f3715ddce55d5194d46e57a222708011201011a2022ecbf124eff995ecf01998dd8346b71810af164e192feeb4d4287085128b9df";
        bytes memory value =
            hex"0a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e31673966716a7a63766a687935336d77797137763432633837613439666d37713772646568386312460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103820c4b94dccd7d74706216c426fe884d9a4404410df69d6421899595c5a9c12218012001";
        IbcCoreCommitmentV1MerkleProof.Data memory decoded =
            IbcCoreCommitmentV1MerkleProof.decode(proof);

        (
            UnionIcs23.ExistenceProof memory existProof,
            UnionIcs23.ExistenceProof memory existProof2
        ) = getExists(decoded);

        bytes memory encoded = abi.encode([existProof, existProof2]);
        ICS23MembershipVerifier verifier = new ICS23MembershipVerifier();
        vm.resumeGasMetering();
        assertTrue(
            verifier.verifyMembership(
                root,
                encoded,
                bytes("acc"),
                hex"014152090B0C95C948EDC407995560FEED4A9DF81E",
                value
            )
        );
    }

    function test_connectionExist() public {
        vm.pauseGasMetering();
        bytes memory root =
            hex"899CD0B55A4FEDE9AF3C959C43ED3AE6805293642590A81CD95B4C97F89CC424";
        bytes memory proof =
            hex"0abc020ab9020a18636f6e6e656374696f6e732f636f6e6e656374696f6e2d31125b0a0930382d7761736d2d3112230a0131120d4f524445525f4f524445524544120f4f524445525f554e4f524445524544180222250a0e636f6d6574626c732d6e65772d30120c636f6e6e656374696f6e2d301a050a0369626328061a0c0801180120012a040002f006222c080112050204f006201a212075c4910f51207d3c65960120fe931f138e2624668d75869f51b8442593dd5eab222a080112260408de0a2002b6fcf07091245d162f1196b003c555c564980e02c4d4a9fa0a249798f4b25e20222c08011205060ede0a201a2120ff6b0a04e076eecbabfee4e751c0523cbedba898211b5847404e2d954a2203e3222a08011226081ede0a20635053419cfb6a81c839860d99f3ed002840124a790ddd9f066d8bce63f9df54200afc010af9010a03696263122024b15e198bcf648dee62c7ca1fd8c3950c85c3d898833180c3e3c412ccbc559d1a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc222508011221016376cbd7b917c7105ddac35bdeddd79e6c9cbbc66dd227941599de2b9bc8b3de222708011201011a200d68ac7c3e8daf94c65ccdfe5b7397f50e80325240ef9b2a0ec483afaea30544";
        bytes memory value =
            hex"0a0930382d7761736d2d3112230a0131120d4f524445525f4f524445524544120f4f524445525f554e4f524445524544180222250a0e636f6d6574626c732d6e65772d30120c636f6e6e656374696f6e2d301a050a036962632806";
        IbcCoreCommitmentV1MerkleProof.Data memory decoded =
            IbcCoreCommitmentV1MerkleProof.decode(proof);
        (
            UnionIcs23.ExistenceProof memory existProof,
            UnionIcs23.ExistenceProof memory existProof2
        ) = getExists(decoded);

        bytes memory encoded = abi.encode([existProof, existProof2]);
        ICS23MembershipVerifier verifier = new ICS23MembershipVerifier();
        vm.resumeGasMetering();
        assertTrue(
            verifier.verifyMembership(
                root,
                encoded,
                bytes("ibc"),
                bytes("connections/connection-1"),
                value
            )
        );
    }

    function test_clientStateExist() public {
        vm.pauseGasMetering();
        bytes memory root =
            hex"971AF378C1F256110B3BA2EFD90325D5B5AFC8185997F2C12A7C4638B906CC2F";
        bytes memory proof =
            hex"0ab0030aad030a1d636c69656e74732f30382d7761736d2d312f636c69656e74537461746512c7010a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e7011a0c0801180120012a040002ae06222c080112050204b006201a2120980ab410769397da376376a2756754b225f34cc0eea404b068924f64180abcc4222c080112050408b006201a21209d79cf7fc2f248ea0a56cff266ac54cfbc06687e25ffee99aec2884856d0104f222a080112260610b006203e808c2bc895d44d05d7af6d8b0424fabb1d9ab6f53b10cdb084b2996f75bfa620222c08011205081eb006201a212095bb7de983d8ea1282a2d60e2f6c675bec25f82be86aa874ff0f15827c1ab3ed0afc010af9010a036962631220859b7ac80b1c0ca82504e0d8e9de460d42ca66a03e708cbd09869e5216c73a591a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc22250801122101c9d0a585c82dc572f3fcedc70302d4c3fbbc9e84f0618c6b446a70efa312e8dc222708011201011a20952029410a533cf530124179204303bea59a86f5b4993291c5b8ca406412c5f7";
        bytes memory value =
            hex"0a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e701";
        IbcCoreCommitmentV1MerkleProof.Data memory decoded =
            IbcCoreCommitmentV1MerkleProof.decode(proof);
        (
            UnionIcs23.ExistenceProof memory existProof,
            UnionIcs23.ExistenceProof memory existProof2
        ) = getExists(decoded);

        bytes memory encoded = abi.encode([existProof, existProof2]);
        ICS23MembershipVerifier verifier = new ICS23MembershipVerifier();
        vm.resumeGasMetering();
        assertTrue(
            verifier.verifyMembership(
                root,
                encoded,
                bytes("ibc"),
                bytes("clients/08-wasm-1/clientState"),
                value
            )
        );
    }

    function test_existenceProofRootMismatch() public {
        vm.pauseGasMetering();
        bytes memory root =
            hex"971AF378C1F256110B3BA2EFD90325D5B5AFC8185997F2C12A7C4638B906CC22";
        bytes memory proof =
            hex"0ab0030aad030a1d636c69656e74732f30382d7761736d2d312f636c69656e74537461746512c7010a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e7011a0c0801180120012a040002ae06222c080112050204b006201a2120980ab410769397da376376a2756754b225f34cc0eea404b068924f64180abcc4222c080112050408b006201a21209d79cf7fc2f248ea0a56cff266ac54cfbc06687e25ffee99aec2884856d0104f222a080112260610b006203e808c2bc895d44d05d7af6d8b0424fabb1d9ab6f53b10cdb084b2996f75bfa620222c08011205081eb006201a212095bb7de983d8ea1282a2d60e2f6c675bec25f82be86aa874ff0f15827c1ab3ed0afc010af9010a036962631220859b7ac80b1c0ca82504e0d8e9de460d42ca66a03e708cbd09869e5216c73a591a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc22250801122101c9d0a585c82dc572f3fcedc70302d4c3fbbc9e84f0618c6b446a70efa312e8dc222708011201011a20952029410a533cf530124179204303bea59a86f5b4993291c5b8ca406412c5f7";
        bytes memory value =
            hex"0a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e701";
        IbcCoreCommitmentV1MerkleProof.Data memory decoded =
            IbcCoreCommitmentV1MerkleProof.decode(proof);
        (
            UnionIcs23.ExistenceProof memory existProof,
            UnionIcs23.ExistenceProof memory existProof2
        ) = getExists(decoded);

        bytes memory encoded = abi.encode([existProof, existProof2]);
        ICS23MembershipVerifier verifier = new ICS23MembershipVerifier();
        vm.resumeGasMetering();
        assertFalse(
            verifier.verifyMembership(
                root,
                encoded,
                bytes("ibc"),
                bytes("clients/08-wasm-1/clientState"),
                value
            )
        );
    }

    function test_existenceProofKeyMismatch() public {
        vm.pauseGasMetering();
        bytes memory root =
            hex"971AF378C1F256110B3BA2EFD90325D5B5AFC8185997F2C12A7C4638B906CC2F";
        bytes memory proof =
            hex"0ab0030aad030a1d636c69656e74732f30382d7761736d2d312f636c69656e74537461746512c7010a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e7011a0c0801180120012a040002ae06222c080112050204b006201a2120980ab410769397da376376a2756754b225f34cc0eea404b068924f64180abcc4222c080112050408b006201a21209d79cf7fc2f248ea0a56cff266ac54cfbc06687e25ffee99aec2884856d0104f222a080112260610b006203e808c2bc895d44d05d7af6d8b0424fabb1d9ab6f53b10cdb084b2996f75bfa620222c08011205081eb006201a212095bb7de983d8ea1282a2d60e2f6c675bec25f82be86aa874ff0f15827c1ab3ed0afc010af9010a036962631220859b7ac80b1c0ca82504e0d8e9de460d42ca66a03e708cbd09869e5216c73a591a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc22250801122101c9d0a585c82dc572f3fcedc70302d4c3fbbc9e84f0618c6b446a70efa312e8dc222708011201011a20952029410a533cf530124179204303bea59a86f5b4993291c5b8ca406412c5f7";
        bytes memory value =
            hex"0a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e701";
        IbcCoreCommitmentV1MerkleProof.Data memory decoded =
            IbcCoreCommitmentV1MerkleProof.decode(proof);
        (
            UnionIcs23.ExistenceProof memory existProof,
            UnionIcs23.ExistenceProof memory existProof2
        ) = getExists(decoded);

        bytes memory encoded = abi.encode([existProof, existProof2]);
        ICS23MembershipVerifier verifier = new ICS23MembershipVerifier();
        vm.resumeGasMetering();
        assertFalse(
            verifier.verifyMembership(
                root,
                encoded,
                bytes("ibc"),
                bytes("clients/08-wasm-2/clientState"),
                value
            )
        );
    }

    function test_existenceProofValueMismatch() public {
        vm.pauseGasMetering();
        bytes memory root =
            hex"971AF378C1F256110B3BA2EFD90325D5B5AFC8185997F2C12A7C4638B906CC2F";
        bytes memory proof =
            hex"0ab0030aad030a1d636c69656e74732f30382d7761736d2d312f636c69656e74537461746512c7010a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e7011a0c0801180120012a040002ae06222c080112050204b006201a2120980ab410769397da376376a2756754b225f34cc0eea404b068924f64180abcc4222c080112050408b006201a21209d79cf7fc2f248ea0a56cff266ac54cfbc06687e25ffee99aec2884856d0104f222a080112260610b006203e808c2bc895d44d05d7af6d8b0424fabb1d9ab6f53b10cdb084b2996f75bfa620222c08011205081eb006201a212095bb7de983d8ea1282a2d60e2f6c675bec25f82be86aa874ff0f15827c1ab3ed0afc010af9010a036962631220859b7ac80b1c0ca82504e0d8e9de460d42ca66a03e708cbd09869e5216c73a591a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc22250801122101c9d0a585c82dc572f3fcedc70302d4c3fbbc9e84f0618c6b446a70efa312e8dc222708011201011a20952029410a533cf530124179204303bea59a86f5b4993291c5b8ca406412c5f7";
        bytes memory value =
            hex"0a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e700";
        IbcCoreCommitmentV1MerkleProof.Data memory decoded =
            IbcCoreCommitmentV1MerkleProof.decode(proof);
        (
            UnionIcs23.ExistenceProof memory existProof,
            UnionIcs23.ExistenceProof memory existProof2
        ) = getExists(decoded);

        bytes memory encoded = abi.encode([existProof, existProof2]);
        ICS23MembershipVerifier verifier = new ICS23MembershipVerifier();
        vm.resumeGasMetering();
        assertFalse(
            verifier.verifyMembership(
                root,
                encoded,
                bytes("ibc"),
                bytes("clients/08-wasm-1/clientState"),
                value
            )
        );
    }

    function getNonExists(IbcCoreCommitmentV1MerkleProof.Data memory decoded)
        internal
        returns (
            UnionIcs23.NonExistenceProof memory,
            UnionIcs23.ExistenceProof memory
        )
    {
        UnionIcs23.NonExistenceProof memory nonExistProof;
        nonExistProof.key = decoded.proofs[0].nonexist.key;

        UnionIcs23.ExistenceProof memory leftExistProof;
        leftExistProof.key = decoded.proofs[0].nonexist.left.key;
        leftExistProof.value = decoded.proofs[0].nonexist.left.value;
        leftExistProof.leafPrefix = decoded.proofs[0].nonexist.left.leaf.prefix;

        leftExistProof.path = new UnionIcs23.InnerOp[](
            decoded.proofs[0].nonexist.left.path.length
        );
        for (uint256 i = 0; i < leftExistProof.path.length; i++) {
            leftExistProof.path[i].prefix =
                decoded.proofs[0].nonexist.left.path[i].prefix;
            leftExistProof.path[i].suffix =
                decoded.proofs[0].nonexist.left.path[i].suffix;
        }

        UnionIcs23.ExistenceProof memory rightExistProof;
        rightExistProof.key = decoded.proofs[0].nonexist.right.key;
        rightExistProof.value = decoded.proofs[0].nonexist.right.value;
        rightExistProof.leafPrefix =
            decoded.proofs[0].nonexist.right.leaf.prefix;

        rightExistProof.path = new UnionIcs23.InnerOp[](
            decoded.proofs[0].nonexist.right.path.length
        );
        for (uint256 i = 0; i < rightExistProof.path.length; i++) {
            rightExistProof.path[i].prefix =
                decoded.proofs[0].nonexist.right.path[i].prefix;
            rightExistProof.path[i].suffix =
                decoded.proofs[0].nonexist.right.path[i].suffix;
        }

        nonExistProof.left = leftExistProof;
        nonExistProof.right = rightExistProof;

        UnionIcs23.ExistenceProof memory existProof2;
        existProof2.key = decoded.proofs[1].exist.key;
        existProof2.value = decoded.proofs[1].exist.value;
        existProof2.leafPrefix = decoded.proofs[1].exist.leaf.prefix;

        existProof2.path =
            new UnionIcs23.InnerOp[](decoded.proofs[1].exist.path.length);
        for (uint256 i = 0; i < existProof2.path.length; i++) {
            existProof2.path[i].prefix = decoded.proofs[1].exist.path[i].prefix;
            existProof2.path[i].suffix = decoded.proofs[1].exist.path[i].suffix;
        }

        return (nonExistProof, existProof2);
    }

    function test_accountNonExistence() public {
        vm.pauseGasMetering();
        bytes memory root =
            hex"C9A25B954FEF48EC601359591A28C9A2FD32A411421AEF2DC16DC8A68B3CFA98";
        bytes memory proof =
            hex"0a96061293060a15014152090b0c95c948edc407995560feed4a9df88812fa020a15014152090b0c95c948edc407995560feed4a9df81e129e010a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e31673966716a7a63766a687935336d77797137763432633837613439666d37713772646568386312460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103820c4b94dccd7d74706216c426fe884d9a4404410df69d6421899595c5a9c122180420031a0b0801180120012a0300027822290801122502047820170c890f01b9fa9ab803511bbc7be7c25359309f04d021a72e0a9b93b8ff72c020222c0801120504089601201a21205f282a80f1d186fa1f7b237f81e8bc9a4bb40d5a03cbbdffdd421b1ad4cb16f4222c0801120506109601201a2120e9c65294b7106c7323dcabe4532232c319afc78cd373e338f12df43f8ecfa909222c080112050a309601201a2120a95af7890dba33514ea28a3db7b409f4887b058d6d1e43960c4cd45bb1d9bef81afc020a150143e46d91544517a037a8029b6c7f86f62bab389b129e010a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e3167306a786d79323567357436716461677132646b636c7578376334366b77796d38646563667712460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21034611ea6606f6241fdeb0db1854a785eaa2fef5770694237daaf46057cadb3903180320031a0c0801180120012a0400029601222c0801120502049601201a2120532543090d1564b206e953fd6f97000d9b78bd5a8a424f551d483a58b3f54c57222a0801122604089601207e55a1ee8006e9c29c895a8de8ea8cdc6aaddc10e05ea3d3ee8fac786a73c02d20222c0801120506109601201a2120e9c65294b7106c7323dcabe4532232c319afc78cd373e338f12df43f8ecfa909222c080112050a309601201a2120a95af7890dba33514ea28a3db7b409f4887b058d6d1e43960c4cd45bb1d9bef80a80020afd010a0361636312205281c416bf4f80b9d99428a09f91ff311968a3b2adb199342d63c9db20a417e91a090801180120012a010022250801122101ba30cf8122e71a87fea08d0da9499e0373495a64e1648de8f08ca1a73e1fc1a8222708011201011a203489cd05a389a1d165f19003cea0994df9e55a5cb53b3d659417040be528b86d222708011201011a20e5c60ddccacb1c6b0be7957e8d7a86dc0f8bcec91c91d666d39eb1ebedd1bdf1222708011201011a2047a4c9a64496594e8b255443aa979293b2c7120150cf31e0eeeb8a2a987fd7e8222708011201011a2053bca15bed4becbdfd1b4cd0e63bd3845646022a99a2289a6678d8608f092207";
        IbcCoreCommitmentV1MerkleProof.Data memory decoded =
            IbcCoreCommitmentV1MerkleProof.decode(proof);
        (
            UnionIcs23.NonExistenceProof memory nonExistProof,
            UnionIcs23.ExistenceProof memory existProof
        ) = getNonExists(decoded);

        bytes memory encoded = abi.encode(nonExistProof, existProof);
        ICS23MembershipVerifier verifier = new ICS23MembershipVerifier();
        vm.resumeGasMetering();
        assertTrue(
            verifier.verifyNonMembership(
                root,
                encoded,
                bytes("acc"),
                hex"014152090b0c95c948edc407995560feed4a9df888"
            )
        );
    }
}
