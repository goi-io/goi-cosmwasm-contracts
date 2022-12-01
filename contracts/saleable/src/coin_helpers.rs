use cosmwasm_std::Coin;

use crate::error::SaleableItemError;

pub fn assert_sent_sufficient_coin(
    sent: &[Coin],
    required: Option<Coin>,
) -> Result<(), SaleableItemError> {
    if let Some(required_coin) = required {
        let required_amount = required_coin.amount.u128();
        if required_amount > 0 {
            let sent_sufficient_funds = sent.iter().any(|coin| {
                // check if a given sent coin matches denom
                // and has sufficient amount
                coin.denom == required_coin.denom && coin.amount.u128() >= required_amount
            });

            if sent_sufficient_funds {
                return Ok(());
            } else {
                return Err(SaleableItemError::InsufficientFundsSend {});
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use cosmwasm_std::{coin, coins};

    use crate::error::SaleableItemError;

    use super::*;

    #[test]
    fn assert_sent_sufficient_coin_works() {
        match assert_sent_sufficient_coin(&[], Some(coin(0, "token"))) {
            Ok(()) => {}
            Err(e) => panic!("Unexpected error: {:?}", e),
        };

        match assert_sent_sufficient_coin(&[], Some(coin(5, "token"))) {
            Ok(()) => panic!("Should have raised insufficient funds error"),
            Err(SaleableItemError::InsufficientFundsSend {}) => {}
            Err(e) => panic!("Unexpected error: {:?}", e),
        };

        match assert_sent_sufficient_coin(&coins(10, "smokin"), Some(coin(5, "token"))) {
            Ok(()) => panic!("Should have raised insufficient funds error"),
            Err(SaleableItemError::InsufficientFundsSend {}) => {}
            Err(e) => panic!("Unexpected error: {:?}", e),
        };

        match assert_sent_sufficient_coin(&coins(10, "token"), Some(coin(5, "token"))) {
            Ok(()) => {}
            Err(e) => panic!("Unexpected error: {:?}", e),
        };

        let sent_coins = vec![coin(2, "smokin"), coin(5, "token"), coin(1, "earth")];
        match assert_sent_sufficient_coin(&sent_coins, Some(coin(5, "token"))) {
            Ok(()) => {}
            Err(e) => panic!("Unexpected error: {:?}", e),
        };
    }
}
