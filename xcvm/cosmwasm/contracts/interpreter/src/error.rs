use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
	#[error("{0}")]
	Std(#[from] StdError),

	#[error("Invalid call payload")]
	InvalidCallPayload,

	#[error("Data cannot be serialized")]
	DataSerializationError,

<<<<<<< HEAD
    #[error("Transfer amount cannot be 0")]
    ZeroTransferAmount,
=======
	#[error("A program tag must be a correct utf8 encoded string")]
	InvalidProgramTag,
>>>>>>> 9951df25a7 (chore(style): apply cargofmt)
}
