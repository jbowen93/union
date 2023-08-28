pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import "../../contracts/proto/cosmos/ics23/v1/proofs.sol";
import "../../contracts/proto/ibc/core/commitment/v1/commitment.sol";
import "../../contracts/lib/ICS23.sol";
import "../../contracts/lib/CometblsHelp.sol";

contract ICS23Test is Test {
    function testDecode() public {
        bytes
            memory proof = hex"0acf0112cc010a0441413d3d1ac3010a20636c69656e74732f30392d6c6f63616c686f73742f636c69656e74537461746512350a2a2f6962632e6c69676874636c69656e74732e6c6f63616c686f73742e76322e436c69656e74537461746512070a05080210d9021a0c0801180120012a040002b205222c080112050204b205201a212075c4910f51207d3c65960120fe931f138e2624668d75869f51b8442593dd5eab222c08011205060ab205201a2120c617eef82d350859fc4c1b0118079fad20ecd1e5fe2294c86a86e94c0d14ca550afe010afb010a03696263122016df45c79ca168d31360706db2979a792fcd9c2371478a77af951f39ec59487d1a090801180120012a0100222708011201011a20c505c0fd48b1cf2b65619f12b3144e19c60b4e6b62525ee936be93d041623ebf222708011201011a20dd55097e1841a5872eb9dbdf5ca65b54143f01aa24b598538786884a824191602225080112210182c1cabef1a0591f8016c06a396858762bb949a7dfc03e568b4273b3c205273d222508011221010d9399f8bc11620accd23dbfaeca52b7cda51a92ec71c044cd1d39a34bea2826222708011201011a2022ecbf124eff995ecf01998dd8346b71810af164e192feeb4d4287085128b9df";

        CosmosIcs23V1CommitmentProof.Data
            memory commitmentProof = CosmosIcs23V1CommitmentProof.decode(proof);
    }

    function ensureNonExistant(
        bytes memory proof,
        bytes memory root,
        bytes memory key
    ) internal {
        CosmosIcs23V1CommitmentProof.Data
            memory commitmentProof = CosmosIcs23V1CommitmentProof.decode(proof);

        CosmosIcs23V1ProofSpec.Data memory tendermintProofSpec = Ics23
            .getTendermintProofSpec();

        Ics23.VerifyNonMembershipError result = Ics23.verifyNonMembership(
            tendermintProofSpec,
            root,
            commitmentProof,
            key
        );

        if (result == Ics23.VerifyNonMembershipError.NonExistenceProofIsNil) {
            revert("non existence proof is nil");
        } else if (result == Ics23.VerifyNonMembershipError.ProofVerify) {
            revert("proof verify failed");
        }

        assert(result == Ics23.VerifyNonMembershipError.None);
    }

    // https://github.com/cosmos/ics23/blob/b1abd8678aab07165efd453c96796a179eb3131f/testdata/tendermint/nonexist_left.json
    function testNonExistLeft() public {
        bytes
            memory proof = hex"12e4030a04010101011adb030a143032615465366472483456706f4f583245507137121e76616c75655f666f725f3032615465366472483456706f4f5832455071371a090801180120012a0100222708011201011a20b843481496dc10561056b63ec8f726f3357395b610355b25082f5768b2073e91222708011201011a20d5281fdd872060e89173d4de1100fa6c96f778467df66abb10cf3b1f5821f182222708011201011a20eb981020433d929c6275ad772accf2e6aa916db97e31d2f26d0b6b07b444bbef222708011201011a204a40e813132aff60b64ba9d109548ab39459ad48a203ab8d3455dd842a7ab1da222708011201011a208f354a84ce1476e0b9cca92e65301a6435b1f242c2f53f943b764a4f326a71c7222708011201011a20ac6451617a6406005035dddad36657fde5312cc4d67d69ca1464611847c10cfb222708011201011a2023c1d1dd62002a0e2efcc679196589a4337234dcd209cb449cc3ac10773b60e0222708011201011a203b11c267328ba761ddc630dd5ef7642aeda05f180539fe93c0ca57729705bc46222708011201011a205ff2e1933be704539463c264b157ff2b8d9960813bd36c69c5208d57e3b1e07e222708011201011a20c4a79e6c0cbf60fb8e5bf940db4c444b7e442951b69c840db38cf28c8aa008be";
        bytes
            memory root = hex"4e2e78d2da505b7d0b00fda55a4b048eed9a23a7f7fc3d801f20ce4851b442aa";
        bytes memory key = hex"01010101";
        ensureNonExistant(proof, root, key);
    }

    // https://github.com/cosmos/ics23/blob/b1abd8678aab07165efd453c96796a179eb3131f/testdata/tendermint/nonexist_middle.json
    function testNonExistMiddle() public {
        bytes
            memory proof = hex"12c0070a14544f31483668784a4b667136547a56767649ffff12cf030a14544f31483668784a4b667136547a567676497747121e76616c75655f666f725f544f31483668784a4b667136547a5676764977471a090801180120012a01002225080112210143e19cb5e5dab017734caa78a2e2bccbb4797b7dc5a91abeab630c66fa6b162522250801122101b575404a1bb42b0fef8ae7f217af88aec769f7d66b5bc4b2913e74d651365473222508011221017c22dc50e866f9a1dce517ea01621161cecd70f4bdcd024b5a392746a1c8dc2622250801122101578105344f2c98c323ba0b8ca31e75aaa2b865cc389681e300b14d1c20713796222708011201011a20895c070c14546ecef7f5cb3a4bda1fd436a0ff99190f90bd037cbeaf52b2ffc1222708011201011a20f7571fca06ac4387c3eae5469c152427b797abb55fa98727eacbd5c1c91b5fb4222508011221015056e6472f8e5c5c9b8881c5f0e49601e9eca31f3e1766aa69c2dc9c6d9112be222708011201011a206c74439556c5edb5aa693af410d3718dbb613d37799f2f4e8ff304a8bfe3351b22250801122101253014334c7b8cd78436979554f7890f3dc1c971925ea31b48fc729cd179c701222708011201011a20b81c19ad4b5d8d15f716b91519bf7ad3d6e2289f9061fd2592a8431ea97806fe1ad5030a14544f433344683150664f76657538585166635778121e76616c75655f666f725f544f433344683150664f766575385851666357781a090801180120012a0100222708011201011a20415d4cfaed0bfc98ac32acc219a8517bfa1983a15cc742e8b2f860167484bd46222708011201011a2098d853d9cc0ee1d2162527f660f2b90ab55b13e5534f1b7753ec481d7901d3ec222708011201011a20b5113e6000c5411b7cfa6fd09b6752a43de0fcd3951ed3b154d162deb53224a2222708011201011a208ce18cd72cc83511cb8ff706433f2fa4208c85b9f4c8d0ed71a614f24b89ae6c22250801122101c611244fe6b5fda4257615902eb24c14efcd9708c7c875d1ac5e867767aa1eab222708011201011a20f7571fca06ac4387c3eae5469c152427b797abb55fa98727eacbd5c1c91b5fb4222508011221015056e6472f8e5c5c9b8881c5f0e49601e9eca31f3e1766aa69c2dc9c6d9112be222708011201011a206c74439556c5edb5aa693af410d3718dbb613d37799f2f4e8ff304a8bfe3351b22250801122101253014334c7b8cd78436979554f7890f3dc1c971925ea31b48fc729cd179c701222708011201011a20b81c19ad4b5d8d15f716b91519bf7ad3d6e2289f9061fd2592a8431ea97806fe";
        bytes
            memory root = hex"4bf28d948566078c5ebfa86db7471c1541eab834f539037075b9f9e3b1c72cfc";
        bytes memory key = hex"544f31483668784a4b667136547a56767649ffff";
        ensureNonExistant(proof, root, key);
    }

    // https://github.com/cosmos/ics23/blob/b1abd8678aab07165efd453c96796a179eb3131f/testdata/tendermint/nonexist_right.json
    function testNonExistRight() public {
        bytes
            memory proof = hex"12a9030a04ffffffff12a0030a147a774e4d4a456f7932674253586277666e63504a121e76616c75655f666f725f7a774e4d4a456f7932674253586277666e63504a1a090801180120012a01002225080112210178a215355c17371583418df95773476b347a853f6eae317677721e0c24e78ad2222508011221015e2cf893e7cd70251eb4debd855c8c9a92f6e0a1fd931cf41e0575846ab174e822250801122101414bae883f8133f0201a2791dafeaef3daa24a6631b3f9402de3a4dc658fd035222508011221012e2829beee266a814af4db08046f4575b011e5ec9d2d93c1510c3cc7d8219edc22250801122101f8286597078491ae0ef61264c218c6e167e4e03f1de47945d9ba75bb41deb81a22250801122101dea6a53098d11ce2138cbcae26b392959f05d7e1e24b9547584571012280f289222508011221010a8e535094d18b2120c38454b445d9accf3f1b255690e6f3d48164ae73b4c775222508011221012cbb518f52ec1f8e26dd36587f29a6890a11c0dd3f94e7a28546e695f296d3a722250801122101839d9ddd9dadf41c0ecfc3f7e20f57833b8fb5bcb703bef4f97910bbe5b579b9";
        bytes
            memory root = hex"83952b0b17e64c862628bcc1277e7f8847589af794ed5a855339281d395ec04f";
        bytes memory key = hex"ffffffff";
        ensureNonExistant(proof, root, key);
    }

    function ensureExistant(
        bytes memory proof,
        bytes memory root,
        bytes memory key,
        bytes memory value
    ) internal {
        CosmosIcs23V1CommitmentProof.Data
            memory commitmentProof = CosmosIcs23V1CommitmentProof.decode(proof);

        CosmosIcs23V1ProofSpec.Data memory tendermintProofSpec = Ics23
            .getTendermintProofSpec();

        Ics23.VerifyMembershipError result = Ics23.verifyMembership(
            tendermintProofSpec,
            root,
            commitmentProof,
            key,
            value
        );

        if (result == Ics23.VerifyMembershipError.ExistenceProofIsNil) {
            revert("existence proof is nil");
        } else if (result == Ics23.VerifyMembershipError.ProofVerify) {
            revert("proof verify failed");
        }

        assert(result == Ics23.VerifyMembershipError.None);
    }

    // https://github.com/cosmos/ics23/blob/b1abd8678aab07165efd453c96796a179eb3131f/testdata/tendermint/exist_left.json
    function testExistLeft() public {
        bytes
            memory proof = hex"0adb030a14303142424373615a55715146735259436c6a5767121e76616c75655f666f725f303142424373615a55715146735259436c6a57671a090801180120012a0100222708011201011a20cb3131cd98b069efcc0e8c7e68da47370adbff32266d7fcd1b0580fdf3961266222708011201011a2021d1205c1f8537205e8fb4b176f960b459d9131669968d59c456442f7673b68b222708011201011a20b82a0e7f4434b3cedb87ea83eb5a70c7dc664c77b2fe21c6245f315e58fdf745222708011201011a20bf0657a0e6fbd8f2043eb2cf751561adcf50547d16201224133eeb8d38145229222708011201011a206d47c03df91a4a0252055d116439d34b5b73f3a24d5cb3cf0d4b08caa540cac4222708011201011a20d5d2926993fa15c7410ac4ee1f1d81afddfb0ab5f6f4706b05f407bc01638149222708011201011a20540719b26a7301ad012ac45ebe716679e5595e5570d78be9b6da8d8591afb374222708011201011a20fccaaa9950730e80b9ccf75ad2cfeab26ae750b8bd6ac1ff1c7a7502f3c64be2222708011201011a20ecb61a6d70accb79c2325fb0b51677ed1561c91af5e10578c8294002fbb3c21e222708011201011a201b3bc1bd8d08af9f6199de84e95d646570cbd9b306a632a5acf617cbd7d1ab0a";
        bytes
            memory root = hex"c569a38a5775bbda2051c34ae00894186f837c39d11dca55495b9aed14f17ddf";
        bytes memory key = hex"303142424373615a55715146735259436c6a5767";
        bytes
            memory value = hex"76616c75655f666f725f303142424373615a55715146735259436c6a5767";
        ensureExistant(proof, root, key, value);
    }

    // https://github.com/cosmos/ics23/blob/b1abd8678aab07165efd453c96796a179eb3131f/testdata/tendermint/exist_middle.json
    function testExistMiddle() public {
        bytes
            memory proof = hex"0ad1030a14513334656d766f39447145585735325257523835121e76616c75655f666f725f513334656d766f394471455857353252575238351a090801180120012a010022250801122101e231d775380f2d663651e213cc726660e2ce0a2f2e9ee12cbb7df32294104a8c222708011201011a2014af194c63500236e52cc290ab24244fab39a520ece7e20fa93f4c9ff80c6626222508011221017966d2ead34418db2eaa04c0dffb9316805e8a0d421d1270c8954c35ee3221382225080112210172339e20a49bb16795a99bd905b47f99c45e5e5a9e6b7fb223dc8fe6751e1bda222708011201011a2053dd1ecc25ff906a0ef4db37ee068f3d8ad6d1d49913eefb847a675a681c5ffa222708011201011a20de90f9951a19497be7e389e02aa79e26faf77080e740e8743249a17a537f287d22250801122101ad4e53e981afc5a71e34ab0c4ffbccf1b468414d9d0939bd08edbd2461bc944a222708011201011a209b4cf89c3995b9dd66d58ab088846b2c6b59c52c6d10ec1d759ca9e9aa5eef5c222508011221013928a078bd66ab3949f5b1846b6d354dbdc1968a416607c7d91555ca26716667222708011201011a20d2d82cf8915b9ae6f92c7eae343e37d312ace05e654ce47acdf57d0a5490b873";
        bytes
            memory root = hex"494b16e3a64a85df143b2881bdd3ec94c3f8e18b343e8ff9c2d61afd05d040c8";
        bytes memory key = hex"513334656d766f39447145585735325257523835";
        bytes
            memory value = hex"76616c75655f666f725f513334656d766f39447145585735325257523835";
        ensureExistant(proof, root, key, value);
    }

    // https://github.com/cosmos/ics23/blob/b1abd8678aab07165efd453c96796a179eb3131f/testdata/tendermint/exist_right.json
    function testExistRight() public {
        bytes
            memory proof = hex"0aab020a147a785a4e6b534c64634d655657526c7658456644121e76616c75655f666f725f7a785a4e6b534c64634d655657526c76584566441a090801180120012a0100222508011221012634b831468dbafb1fc61a979c348ff8462da9a7d550191a6afc916ade16cc9922250801122101ab814d419bfc94ee9920d0ce993ce5da011e43613daf4b6f302855760083d7dd222508011221015a1568c73eaeaba567a6b2b2944b0e9a0228c931884cb5942f58ed835b8a7ac522250801122101a171412db5ee84835ef247768914e835ff80b7711e4aa8060871c2667ec3ea2922250801122101f9c2491884de24fb61ba8f358a56b306a8989bd35f1f8a4c8dabce22f703cc14222508011221012f12a6aa6270eff8a1628052938ff5e36cfcc5bf2eaedc0941ee46398ebc7c38";
        bytes
            memory root = hex"f54227f1a7d90aa2bf7931066196fd3072b7fe6b1fbd49d1e26e85a90d9541bb";
        bytes memory key = hex"7a785a4e6b534c64634d655657526c7658456644";
        bytes
            memory value = hex"76616c75655f666f725f7a785a4e6b534c64634d655657526c7658456644";
        ensureExistant(proof, root, key, value);
    }

    // union-testnet-2
    // key = 0x01 + take(20, sha256(account))
    // proof, value = nix run .#uniond -- genstateproof 345 "014152090B0C95C948EDC407995560FEED4A9DF81E" "/store/acc/key" --node https://rpc.0xc0dejug.uno:443
    // path = split('/', '/acc/${key}')
    // root = nix run .#uniond -- query block 346 --node https://rpc.0xc0dejug.uno:443 | jq .block.header.app_hash
    function testTestnetExist() public {
        bytes
            memory root = hex"B802C903BEFE08624832AAF853DBEA4EDE1F7D50E88BEDD6317831F45CC74A3D";
        bytes
            memory proof = hex"0afa020af7020a15014152090b0c95c948edc407995560feed4a9df81e129e010a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e31673966716a7a63766a687935336d77797137763432633837613439666d37713772646568386312460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103820c4b94dccd7d74706216c426fe884d9a4404410df69d6421899595c5a9c122180120011a0b0801180120012a0300020222290801122502040220170c890f01b9fa9ab803511bbc7be7c25359309f04d021a72e0a9b93b8ff72c020222b08011204040802201a2120a89a7b1aedf861a8c6316009af3d19448bfe8834dfb5546c7e1af7f95c3000b4222b08011204061002201a212029347d33c119e85fc1335f43ad17c4a1986ad44c71837158ceffd36e2f38f986222b080112040a3002201a2120e284b7ed0385d018b1ffcd6f33bf6ac575fb7731704d0ae71be278bd8bf5e0b50a80020afd010a03616363122082d7d632a58654a81bb6764379eff4b6e641e96620a12dac0e250e6caf94f7761a090801180120012a010022250801122101ba30cf8122e71a87fea08d0da9499e0373495a64e1648de8f08ca1a73e1fc1a8222708011201011a208a19e0585632ebada293099d24f28707d453266ae7ded6e854dfd8a025c7ce71222708011201011a204a22410f42f7706402b38c460e74d712c95cea8e6e370c691f43c0abf3f4e104222708011201011a20b999d9a62cbd36a843f207580c4802d194e6441f7f3715ddce55d5194d46e57a222708011201011a2022ecbf124eff995ecf01998dd8346b71810af164e192feeb4d4287085128b9df";
        bytes
            memory value = hex"0a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74127a0a2c756e696f6e31673966716a7a63766a687935336d77797137763432633837613439666d37713772646568386312460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103820c4b94dccd7d74706216c426fe884d9a4404410df69d6421899595c5a9c12218012001";
        bytes[] memory path = new bytes[](2);
        path[0] = "acc";
        path[1] = hex"014152090B0C95C948EDC407995560FEED4A9DF81E";
        assert(
            Ics23.verifyChainedMembership(
                IbcCoreCommitmentV1MerkleProof.decode(proof),
                root,
                path,
                value
            ) == Ics23.VerifyChainedMembershipError.None
        );
    }

    function testConnectionExist() public {
        bytes
            memory root = hex"899CD0B55A4FEDE9AF3C959C43ED3AE6805293642590A81CD95B4C97F89CC424";
        bytes
            memory proof = hex"0abc020ab9020a18636f6e6e656374696f6e732f636f6e6e656374696f6e2d31125b0a0930382d7761736d2d3112230a0131120d4f524445525f4f524445524544120f4f524445525f554e4f524445524544180222250a0e636f6d6574626c732d6e65772d30120c636f6e6e656374696f6e2d301a050a0369626328061a0c0801180120012a040002f006222c080112050204f006201a212075c4910f51207d3c65960120fe931f138e2624668d75869f51b8442593dd5eab222a080112260408de0a2002b6fcf07091245d162f1196b003c555c564980e02c4d4a9fa0a249798f4b25e20222c08011205060ede0a201a2120ff6b0a04e076eecbabfee4e751c0523cbedba898211b5847404e2d954a2203e3222a08011226081ede0a20635053419cfb6a81c839860d99f3ed002840124a790ddd9f066d8bce63f9df54200afc010af9010a03696263122024b15e198bcf648dee62c7ca1fd8c3950c85c3d898833180c3e3c412ccbc559d1a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc222508011221016376cbd7b917c7105ddac35bdeddd79e6c9cbbc66dd227941599de2b9bc8b3de222708011201011a200d68ac7c3e8daf94c65ccdfe5b7397f50e80325240ef9b2a0ec483afaea30544";
        bytes
            memory value = hex"0a0930382d7761736d2d3112230a0131120d4f524445525f4f524445524544120f4f524445525f554e4f524445524544180222250a0e636f6d6574626c732d6e65772d30120c636f6e6e656374696f6e2d301a050a036962632806";
        bytes[] memory path = new bytes[](2);
        path[0] = "ibc";
        path[1] = "connections/connection-1";
        assert(
            Ics23.verifyChainedMembership(
                IbcCoreCommitmentV1MerkleProof.decode(proof),
                root,
                path,
                value
            ) == Ics23.VerifyChainedMembershipError.None
        );
    }

    function testClientStateExist() public {
        bytes
            memory root = hex"971AF378C1F256110B3BA2EFD90325D5B5AFC8185997F2C12A7C4638B906CC2F";
        bytes
            memory proof = hex"0ab0030aad030a1d636c69656e74732f30382d7761736d2d312f636c69656e74537461746512c7010a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e7011a0c0801180120012a040002ae06222c080112050204b006201a2120980ab410769397da376376a2756754b225f34cc0eea404b068924f64180abcc4222c080112050408b006201a21209d79cf7fc2f248ea0a56cff266ac54cfbc06687e25ffee99aec2884856d0104f222a080112260610b006203e808c2bc895d44d05d7af6d8b0424fabb1d9ab6f53b10cdb084b2996f75bfa620222c08011205081eb006201a212095bb7de983d8ea1282a2d60e2f6c675bec25f82be86aa874ff0f15827c1ab3ed0afc010af9010a036962631220859b7ac80b1c0ca82504e0d8e9de460d42ca66a03e708cbd09869e5216c73a591a090801180120012a01002225080112210106b99c0d8119ff1edbcbe165d0f19337dbbc080e677c88e57aa2ae767ebf0f0f222708011201011a20aa650406ea0d76e39dd43d2ea6a91e3fdaa1c908fc21a7ca68e5e62cc8115639222508011221016ac3182364d7cdaa1f52a77b6081e070aa29b2d253f3642169693cde336e2bdc22250801122101c9d0a585c82dc572f3fcedc70302d4c3fbbc9e84f0618c6b446a70efa312e8dc222708011201011a20952029410a533cf530124179204303bea59a86f5b4993291c5b8ca406412c5f7";
        bytes
            memory value = hex"0a252f6962632e6c69676874636c69656e74732e7761736d2e76312e436c69656e745374617465129d010a720a20d8ea171f3c94aea21ebc42a1ed61052acf3f9209c00e4efbaaddac09ed9b807818e0fac1950622310a04900000691a060a049000007022060a04900000712a060a049000007232110a040400000010ffffffffffffffffff01280c30203880024204080110034880c2d72f50a0f4a4011220e8dcc770de5a013041588233812f73ac797ec6078b0011cbcbfe49d474f4c1191a051081f2e701";
        bytes[] memory path = new bytes[](2);
        path[0] = "ibc";
        path[1] = "clients/08-wasm-1/clientState";
        assert(
            Ics23.verifyChainedMembership(
                IbcCoreCommitmentV1MerkleProof.decode(proof),
                root,
                path,
                value
            ) == Ics23.VerifyChainedMembershipError.None
        );
    }
}
