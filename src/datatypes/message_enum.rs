use crate::{
    CombinationProduct, DeleteOrder, EquilibriumPrice, Executed, ExecutionWithPriceInfo, LegPrice,
    ProductInfo, PutOrder, SecondTag, SystemEventInfo, TickSize, TradingStatusInfo,
};

macro_rules! dclr_message_enum {
    ($($ident:ident,)*) => {
        pub enum MessageEnum {
            $( $ident($ident), )*
        }

        impl MessageEnum {
            pub fn tag(&self) -> char {
                match self {
                    $( MessageEnum::$ident(_) => $ident::TAG, )*
                }
            }
        }

        $(
            impl TryFrom<MessageEnum> for $ident {
                type Error = &'static str;
                fn try_from(msg_enum: MessageEnum) -> Result<Self, Self::Error> {
                    match msg_enum {
                        MessageEnum::$ident(item) => Ok(item),
                        _ => Err(stringify!($ident))
                    }
                }
            }
        ) *
    };
}

dclr_message_enum!(
    CombinationProduct,
    DeleteOrder,
    EquilibriumPrice,
    Executed,
    ExecutionWithPriceInfo,
    LegPrice,
    ProductInfo,
    PutOrder,
    SecondTag,
    SystemEventInfo,
    TickSize,
    TradingStatusInfo,
);
