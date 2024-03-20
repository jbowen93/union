// https://github.com/rust-lang/rust/issues/35853#issuecomment-415993963
macro_rules! with_dollar_sign {
    ($($body:tt)*) => {
        macro_rules! __with_dollar_sign { $($body)* }
        __with_dollar_sign!($);
    }
}

macro_rules! try_from_relayer_msg {
    (
        chain = $Chain:ty,
        generics = ($($generics:tt)+),
        msgs = $Enum:ident(
            $($Variant:ident($Ty:ty),)+
        ),
    ) => {
        with_dollar_sign! {
            ($d:tt) => {
                macro_rules! with_generics {
                    (
                        chain = $d Chain:ty,
                        msgs = $d Enum:ident(
                            $d ($d Variant:ident($d Ty:ty),)+
                        ),
                    ) => {
                        $d (
                            impl <$($generics)+> TryFrom<queue_msg::QueueMsg<crate::RelayerMsgTypes>> for Identified<$d Chain, Tr, $d Ty>
                            where
                                identified!(Data<$d Chain, Tr>): TryFrom<AnyLightClientIdentified<AnyData>, Error = AnyLightClientIdentified<AnyData>> + Into<AnyLightClientIdentified<AnyData>>
                            {
                                type Error = queue_msg::QueueMsg<crate::RelayerMsgTypes>;
                                fn try_from(value: queue_msg::QueueMsg<crate::RelayerMsgTypes>) -> Result<Identified<$d Chain, Tr, $d Ty>, queue_msg::QueueMsg<crate::RelayerMsgTypes>> {
                                    match value {
                                        queue_msg::QueueMsg::Data(data) => {
                                            let Identified {
                                                chain_id,
                                                t,
                                                __marker: _,
                                            } = data.try_into().map_err(queue_msg::QueueMsg::Data)?;

                                            match t {
                                                crate::Data::LightClientSpecific(
                                                    crate::data::LightClientSpecificData($d Enum::$d Variant(
                                                    t,
                                                ))) => Ok(crate::id(chain_id, t)),
                                                _ => Err(queue_msg::QueueMsg::Data(Into::<AnyLightClientIdentified<AnyData>>::into(crate::id(chain_id, t))))
                                            }

                                        },
                                        _ => Err(value),
                                    }
                                }
                            }

                            impl <$($generics)+> From<Identified<$d Chain, Tr, $d Ty>> for crate::AnyLightClientIdentified<crate::data::AnyData>
                            where
                                AnyLightClientIdentified<AnyData>: From<identified!(Data<$d Chain, Tr>)>
                            {
                                fn from(Identified { chain_id, t, __marker: _ }: Identified<$d Chain, Tr, $d Ty>) -> crate::AnyLightClientIdentified<crate::data::AnyData> {
                                    crate::AnyLightClientIdentified::from(crate::id(
                                        chain_id,
                                        Data::LightClientSpecific(crate::data::LightClientSpecificData($d Enum::$d Variant(
                                            t,
                                        ))),
                                    ))
                                }
                            }

                            impl <$($generics)+> TryFrom<crate::AnyLightClientIdentified<crate::data::AnyData>> for Identified<$d Chain, Tr, $d Ty>
                            where
                                identified!(Data<$d Chain, Tr>): TryFrom<AnyLightClientIdentified<AnyData>, Error = AnyLightClientIdentified<AnyData>> + Into<AnyLightClientIdentified<AnyData>>
                            {
                                type Error = crate::AnyLightClientIdentified<crate::data::AnyData>;

                                fn try_from(value: crate::AnyLightClientIdentified<crate::data::AnyData>) -> Result<Identified<$d Chain, Tr, $d Ty>, crate::AnyLightClientIdentified<crate::data::AnyData>> {
                                    let Identified {
                                        chain_id,
                                        t,
                                        __marker: _,
                                    } = value.try_into()?;

                                    match t {
                                        Data::LightClientSpecific(crate::data::LightClientSpecificData($d Enum::$d Variant(
                                            t,
                                        ))) => Ok(crate::id(chain_id, t)),
                                        _ => Err(Into::<AnyLightClientIdentified<AnyData>>::into(crate::id(chain_id, t)))
                                    }
                                }
                            }
                        )+

                        // impl From<<$d Chain as LightClient>::$d LcMsg> for $d Specific<$d Chain> {
                        //     fn from(msg: <$d Chain as LightClient>::$d LcMsg) -> Self {
                        //         Self(msg)
                        //     }
                        // }
                    };
                }
            }
        }

        with_generics!(
            chain = $Chain,
            msgs = $Enum(
                $($Variant($Ty),)+
            ),
        );
    };
}

// functionality common between all cosmos-sdk chains
pub mod cosmos_sdk;

pub mod cosmos;
pub mod union;

pub mod ethereum;
