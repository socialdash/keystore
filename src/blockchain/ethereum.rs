use rlp;
use std::str::FromStr;

use super::error::*;
use super::utils::{bytes_to_hex, hex_to_bytes};
use super::BlockchainService;
use ethcore_transaction::{Action, Transaction};
use ethereum_types::{H160, U256};
use ethkey::Secret;
use ethkey::{Generator, Random};
use models::*;
use prelude::*;

#[derive(Default)]
pub struct EthereumService {
    stq_gas_limit: usize,
    stq_contract_address: String,
    stq_transfer_method_number: String,
    chain_id: Option<u64>,
}

impl EthereumService {
    pub fn new(stq_gas_limit: usize, stq_contract_address: String, stq_transfer_method_number: String, chain_id: Option<u64>) -> Self {
        EthereumService {
            stq_gas_limit,
            stq_contract_address,
            stq_transfer_method_number,
            chain_id,
        }
    }
}

impl BlockchainService for EthereumService {
    fn generate_key(&self, currency: Currency) -> Result<(PrivateKey, BlockchainAddress), Error> {
        let mut random = Random;
        let pair = random.generate().map_err(ectx!(try ErrorSource::Random, ErrorKind::Internal))?;
        let private_key = PrivateKey::new(format!("{:x}", pair.secret()));
        let blockchain_address = BlockchainAddress::new(format!("{:x}", pair.address()));
        Ok((private_key, blockchain_address))
    }
    fn sign(&self, key: PrivateKey, tx: UnsignedTransaction) -> Result<RawTransaction, Error> {
        let UnsignedTransaction {
            to,
            currency,
            value,
            fee_price,
            nonce: maybe_nonce,
            ..
        } = tx;
        let nonce = maybe_nonce.ok_or(ectx!(try err ErrorKind::MissingNonce, ErrorSource::Signer, ErrorKind::MissingNonce))?;
        let nonce: U256 = nonce.into();
        let gas_price: U256 = fee_price.into();
        let gas: U256 = self.stq_gas_limit.into();
        let tx_value: U256 = match currency {
            Currency::Eth => value.into(),
            Currency::Stq => 0.into(),
            _ => panic!("attempted to sign non-ethereum currency with ethereum algos "),
        };
        let action = match currency {
            Currency::Eth => {
                let to = H160::from_str(&to.clone().into_inner())
                    .map_err(ectx!(try ErrorContext::H160Convert, ErrorKind::MalformedHexString))?;
                Action::Call(to)
            }
            Currency::Stq => {
                let to = H160::from_str(&self.stq_contract_address)
                    .map_err(ectx!(try ErrorContext::H160Convert, ErrorKind::MalformedHexString))?;
                Action::Call(to)
            }
            _ => panic!("attempted to sign non-ethereum currency with ethereum algos "),
        };
        let data = match currency {
            Currency::Eth => Vec::new(),
            Currency::Stq => {
                let mut data: Vec<u8> = Vec::new();
                let method = hex_to_bytes(self.stq_transfer_method_number.clone())?;
                let to = serialize_address(to)?;
                let value = serialize_amount(value);
                data.extend(method.iter());
                data.extend(to.iter());
                data.extend(value.iter());
                data
            }
            _ => panic!("attempted to sign non-ethereum currency with ethereum algos "),
        };

        let tx = Transaction {
            nonce,
            gas_price,
            gas,
            action,
            value: tx_value,
            data,
        };
        let secret = private_key_to_secret(key)?;
        let signed = tx.sign(&secret, self.chain_id);
        let raw_data = rlp::encode(&signed).to_vec();
        let raw_hex_data = bytes_to_hex(&raw_data);
        Ok(RawTransaction::new(raw_hex_data))
    }
}

fn private_key_to_secret(key: PrivateKey) -> Result<Secret, Error> {
    let hex_pk = key.clone().into_inner();
    let bytes = hex_to_bytes(hex_pk)?;
    Secret::from_slice(&bytes)
        .ok_or(ectx!(err ErrorKind::MalformedHexString, ErrorContext::PrivateKeyConvert, ErrorKind::MalformedHexString => key))
}

fn serialize_amount(amount: Amount) -> Vec<u8> {
    to_padded_32_bytes(&amount.to_bytes())
}

fn serialize_address(address: BlockchainAddress) -> Result<Vec<u8>, Error> {
    hex_to_bytes(address.into_inner()).map(|data| to_padded_32_bytes(&data))
}

fn to_padded_32_bytes(data: &[u8]) -> Vec<u8> {
    let zeros_len = 32 - data.len();
    let mut res = Vec::with_capacity(32);
    for _ in 0..zeros_len {
        res.push(0);
    }
    res.extend(data.iter());
    res
}

#[cfg(test)]
mod tests {
    use super::super::BlockchainService;
    use super::*;

    #[test]
    fn test_sign() {
        let ethereum_service = EthereumService {
            stq_gas_limit: 100000,
            stq_contract_address: "1bf2092a42166b2ae19b7b23752e7d2dab5ba91a".to_string(),
            stq_transfer_method_number: "a9059cbb".to_string(),
            chain_id: Some(42),
        };
        let private_key = PrivateKey::new("b3c0e85a511cc6d21423a386de29dcf2cda6b2f2fa5ebb47948401bbb90458db".to_string());
        let to = BlockchainAddress::new("00d44DD2f6a2d2005326Db58eC5137204C5Cba5A".to_string());
        let from = to.clone(); // from is inferred from private_key, so this is abundant for sign method
        let cases = [
            (
                UnsignedTransaction {
                    id: TransactionId::default(),
                    from: from.clone(),
                    to: to.clone(),
                    currency: Currency::Eth,
                    value: Amount::new(25000000000000000000),
                    fee_price: Amount::new(30000000000),
                    nonce: Some(0),
                    utxos: Vec::new(),
                },
                "f86e808506fc23ac00830186a09400d44dd2f6a2d2005326db58ec5137204c5cba5a89015af1d78b58c400008077a09bb23536f025bc054d87c68faf2dcb99141a0be6ab28ea888974d4a9b5d9473ca0436070757106922b3c65c81592d5c8ea55fac876b78b8c5ce946711ff8c74cb4",
            ),
            (
                UnsignedTransaction {
                    id: TransactionId::default(),
                    from: from.clone(),
                    to: to.clone(),
                    currency: Currency::Stq,
                    value: Amount::new(25000000000000000000),
                    fee_price: Amount::new(30000000000),
                    nonce: Some(0),
                    utxos: Vec::new(),
                },
                "f8aa808506fc23ac00830186a0941bf2092a42166b2ae19b7b23752e7d2dab5ba91a80b844a9059cbb00000000000000000000000000d44dd2f6a2d2005326db58ec5137204c5cba5a0000000000000000000000000000000000000000000000015af1d78b58c4000077a0de0e8ba7ed0175250a275d80590dacefd1cdc3fd8e06f931af7e420e88c14f03a07d0330efa45e73d84263af996ad645b117e3531b2ecb6b5f3d1508db5b0aae63",
            ),
        ];
        for case in cases.into_iter() {
            let (input, expected) = case.clone();
            let output = ethereum_service.sign(private_key.clone(), input).unwrap();
            assert_eq!(output, RawTransaction::new(expected.to_string()));
        }
    }
    #[test]
    fn test_serialize_address() {
        let cases = [
            (
                "8A54941dB68A89d63Af5064F53B1C8Fc832B4D89",
                "0000000000000000000000008a54941db68a89d63af5064f53b1c8fc832b4d89",
            ),
            (
                "0054941dB68A89d63Af5064F53B1C8Fc832B4D89",
                "0000000000000000000000000054941db68a89d63af5064f53b1c8fc832b4d89",
            ),
            (
                "0054941dB68A89d63Af5064F53B1C8Fc83010089",
                "0000000000000000000000000054941db68a89d63af5064f53b1c8fc83010089",
            ),
        ];
        for case in cases.into_iter() {
            let (input, expected) = case.clone();
            let address = BlockchainAddress::new(input.to_string());
            let serialized = serialize_address(address).unwrap();
            assert_eq!(bytes_to_hex(&serialized), expected);
        }
    }

    #[test]
    fn test_serialize_amount() {
        let cases = [(180000000000u128, "00000000000000000000000000000000000000000000000000000029e8d60800")];
        for case in cases.into_iter() {
            let (input, expected) = case.clone();
            let amount = Amount::new(input);
            let serialized = serialize_amount(amount);
            assert_eq!(bytes_to_hex(&serialized), expected);
        }
    }

}