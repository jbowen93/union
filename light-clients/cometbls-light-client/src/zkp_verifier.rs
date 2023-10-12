#[deprecated(note = "the circuit has been generalized, use verify_zkp_v2() instead")]
pub fn verify_zkp(
    trusted_validators_hash: &[u8],
    untrusted_validators_hash: &[u8],
    message: &[u8],
    zkp: &[u8],
) -> bool {
    cometbls_groth16_verifier::verify_zkp(
        cometbls_groth16_verifier::testnet_vk(),
        trusted_validators_hash.into(),
        untrusted_validators_hash.into(),
        message,
        zkp,
    )
    .unwrap();
    true
}

pub fn verify_zkp_v2(
    trusted_validators_hash: &[u8],
    untrusted_validators_hash: &[u8],
    message: &[u8],
    zkp: &[u8],
) -> bool {
    cometbls_groth16_verifier::verify_zkp_v2(
        trusted_validators_hash.into(),
        untrusted_validators_hash.into(),
        message,
        zkp,
    )
    .unwrap();
    true
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        assert!(
            super::verify_zkp(
                &hex::decode("1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877").unwrap(),
                &hex::decode("1C693384712792A76DAC1C8E967AEACAD9426A3A2E30513AC201A8F009065877").unwrap(),
                &hex::decode("650802113e0200000000000022480a207022627e60ed78120d2fe8dc7acdb58a2321b0304f8912d2dfb86ce038e23ca812240801122041b8793236ee0980e2eaf1a2fad268c4a3d8979a0c432f06e284eec5e74dd69c320e756e696f6e2d6465766e65742d31").unwrap(),
                &hex::decode("25670583A18A0FA734EE839824AEB2EFAF00F2704178C951B70A01E956C164F32CA7B62707FF3916D88F02F67C1C9334C1EC929F37551212DFCF667903C93C2E0E4D493A02092736D6ADD9A66AAE2B55028FA72FB6137639547BBF4C47EB073E2BB2BE616A4182F3B278C7185E4D21EE535BBA1F44F260D23F869F3E2B3F27400318AAC18834CBDE7001AB47637B05ADDF2C0101CCC1BED2BAB0981AB76225F4212F72E61FED29327F9C81E06DB3C9B67FBF6542BF7742CE807DD0B38134DD652C01BB21CF6B5C01AC3C1E749E9E6859DCD8FAA24C32AC976CD5EF8989E37D6D2896AE7082AC48A94B1BF6BCFCAC412EAD66A22986366C78FA8072060DCC95781159E6255C367EAFBFDAE0C611935C2E6FEEA3F76810FBA9F95FA45700EFA5A017D399707E896688C2CCBB13D014D5189F523D6912AE3D01D0AE5F2EC6B05FA80466F421D4936925454BB6941FD367C93AC498C2CE3503DCB41A58C0C437F39E").unwrap()
            ),
        );
    }
}
