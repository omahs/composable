use crate::{
	pallet_hook::PalletHook,
	runtimes::{
		abstraction::CosmwasmAccount,
		vm::{ContractBackend, CosmwasmVMError, CosmwasmVMShared},
	},
	types::*,
	CodeIdToInfo, Config, ContractToInfo, CurrentNonce, Error, Event, Pallet,
};
use alloc::vec::Vec;
use composable_support::abstractions::utils::increment::Increment;
use core::marker::PhantomData;
use cosmwasm_vm::{
	cosmwasm_std::{Binary, Coin, Event as CosmwasmEvent},
	executor::{
		cosmwasm_call, AsFunctionName, ExecuteCall, InstantiateCall, MigrateCall, ReplyCall,
	},
	system::{
		cosmwasm_system_entrypoint_hook, cosmwasm_system_run_hook, CosmwasmCallVM, CosmwasmCodeId,
		CosmwasmDynamicVM, StargateCosmwasmCallVM,
	},
	vm::VmErrorOf,
};
use cosmwasm_vm_wasmi::WasmiVM;
use frame_support::ensure;
/// Prepares for `instantiate` entrypoint call.
///
/// * `instantiator` - Address of the account that calls this entrypoint.
pub(crate) fn setup_instantiate_call<T: Config>(
	instantiator: AccountIdOf<T>,
	code_id: CosmwasmCodeId,
	salt: &[u8],
	admin: Option<AccountIdOf<T>>,
	label: ContractLabelOf<T>,
	message: &[u8],
) -> Result<DispatchableCall<InstantiateCall, AccountIdOf<T>, T>, Error<T>> {
	let code_hash = CodeIdToInfo::<T>::get(code_id)
		.ok_or(Error::<T>::CodeNotFound)?
		.pristine_code_hash;
	let contract = Pallet::<T>::derive_contract_address(&instantiator, salt, code_hash, message);
	// Make sure that contract address does not already exist
	ensure!(Pallet::<T>::contract_exists(&contract).is_err(), Error::<T>::ContractAlreadyExists);
	let nonce = CurrentNonce::<T>::increment().map_err(|_| Error::<T>::NonceOverflow)?;
	let trie_id = Pallet::<T>::derive_contract_trie_id(&contract, nonce);
	let contract_info =
		ContractInfoOf::<T> { instantiator: instantiator.clone(), code_id, trie_id, admin, label };
	ContractToInfo::<T>::insert(&contract, &contract_info);
	CodeIdToInfo::<T>::try_mutate(code_id, |entry| -> Result<(), Error<T>> {
		let code_info = entry.as_mut().ok_or(Error::<T>::CodeNotFound)?;
		code_info.refcount =
			code_info.refcount.checked_add(1).ok_or(Error::<T>::RefcountOverflow)?;
		Ok(())
	})?;
	Pallet::<T>::deposit_event(Event::<T>::Instantiated {
		contract: contract.clone(),
		info: contract_info,
	});
	Ok(DispatchableCall {
		sender: instantiator,
		contract: contract.clone(),
		entrypoint: EntryPoint::Instantiate,
		output: contract,
		marker: PhantomData,
	})
}

/// Prepares for `execute` entrypoint call.
///
/// * `executor` - Address of the account that calls this entrypoint.
/// * `contract` - Address of the contract to be called.
pub(crate) fn setup_execute_call<T: Config>(
	executor: AccountIdOf<T>,
	contract: AccountIdOf<T>,
) -> Result<DispatchableCall<ExecuteCall, (), T>, Error<T>> {
	Ok(DispatchableCall {
		entrypoint: EntryPoint::Execute,
		sender: executor,
		contract,
		output: (),
		marker: PhantomData,
	})
}

/// Prepares for `reply` entrypoint call.
///
/// * `executor` - Address of the account that calls this entrypoint.
/// * `contract` - Address of the contract to be called.
pub(crate) fn setup_reply_call<T: Config>(
	executor: AccountIdOf<T>,
	contract: AccountIdOf<T>,
) -> Result<DispatchableCall<ReplyCall, (), T>, Error<T>> {
	Ok(DispatchableCall {
		entrypoint: EntryPoint::Reply,
		sender: executor,
		contract,
		output: (),
		marker: PhantomData,
	})
}

/// Prepares for `migrate` entrypoint call.
///
/// * `migrator` - Address of the account that calls this entrypoint.
/// * `contract` - Address of the contract to be called.
/// * `new_code_id` - New code id that the contract will point to (or use).
pub(crate) fn setup_migrate_call<T: Config>(
	shared: &mut CosmwasmVMShared,
	migrator: AccountIdOf<T>,
	contract: AccountIdOf<T>,
	new_code_id: CosmwasmCodeId,
) -> Result<DispatchableCall<MigrateCall, (), T>, Error<T>> {
	let contract_info = Pallet::<T>::contract_info(&contract)?;
	// If the migrate already happened, no need to do that again.
	// This is the case for sub-message execution where `migrate` is
	// called by the VM.
	if contract_info.code_id != new_code_id {
		Pallet::<T>::cosmwasm_call(
			shared,
			migrator.clone(),
			contract.clone(),
			Default::default(),
			|vm| {
				cosmwasm_vm::system::migrate(
					vm,
					CosmwasmAccount::new(migrator.clone()),
					CosmwasmAccount::new(contract.clone()),
					new_code_id,
				)
			},
		)
		.map_err(|_| Error::<T>::NotAuthorized)?;
	}

	Pallet::<T>::deposit_event(Event::<T>::Migrated {
		contract: contract.clone(),
		to: new_code_id,
	});

	Ok(DispatchableCall {
		sender: migrator,
		contract,
		entrypoint: EntryPoint::Migrate,
		output: (),
		marker: PhantomData,
	})
}

/// Generic ready-to-call state for all input types
pub struct DispatchableCall<I, O, T: Config> {
	sender: AccountIdOf<T>,
	contract: AccountIdOf<T>,
	entrypoint: EntryPoint,
	output: O,
	marker: PhantomData<I>,
}

/// Dispatch state for all `Input`s
impl<I, O, T: Config> DispatchableCall<I, O, T> {
	/// Start a cosmwasm transaction by calling an entrypoint.
	///
	/// * `shared` - Shared state of the Cosmwasm VM.
	/// * `funds` - Funds to be transferred before execution.
	/// * `message` - Message to be passed to the entrypoint.
	pub(crate) fn call(
		self,
		shared: &mut CosmwasmVMShared,
		funds: FundsOf<T>,
		message: ContractMessageOf<T>,
	) -> Result<O, CosmwasmVMError<T>>
	where
		for<'x> WasmiVM<DefaultCosmwasmVM<'x, T>>:
			CosmwasmCallVM<I> + CosmwasmDynamicVM<I> + StargateCosmwasmCallVM,
		for<'x> VmErrorOf<WasmiVM<DefaultCosmwasmVM<'x, T>>>:
			From<CosmwasmVMError<T>> + Into<CosmwasmVMError<T>>,
		I: AsFunctionName,
	{
		let entrypoint = self.entrypoint;
		self.call_internal(shared, funds, |vm| {
			cosmwasm_system_entrypoint_hook::<I, _>(vm, &message, |vm, message| {
				match vm.0.contract_runtime {
					ContractBackend::CosmWasm { .. } =>
						cosmwasm_call::<I, _>(vm, message).map(Into::into),
					ContractBackend::Pallet => T::PalletHook::execute(vm, entrypoint, message),
				}
			})
			.map_err(Into::into)
		})
	}

	fn call_internal<F>(
		self,
		shared: &mut CosmwasmVMShared,
		funds: FundsOf<T>,
		message: F,
	) -> Result<O, CosmwasmVMError<T>>
	where
		for<'x> WasmiVM<DefaultCosmwasmVM<'x, T>>:
			CosmwasmCallVM<I> + CosmwasmDynamicVM<I> + StargateCosmwasmCallVM,
		for<'x> VmErrorOf<WasmiVM<DefaultCosmwasmVM<'x, T>>>: Into<CosmwasmVMError<T>>,
		F: for<'x> FnOnce(
			&'x mut WasmiVM<DefaultCosmwasmVM<'x, T>>,
		) -> Result<(Option<Binary>, Vec<CosmwasmEvent>), CosmwasmVMError<T>>,
	{
		Pallet::<T>::do_extrinsic_dispatch(
			shared,
			self.entrypoint,
			self.sender,
			self.contract,
			funds,
			|vm| message(vm).map_err(Into::into),
		)?;
		Ok(self.output)
	}

	/// Continue the execution by running an entrypoint. This is used for running
	/// submessages.
	///
	/// * `shared` - Shared state of the Cosmwasm VM.
	/// * `funds` - Funds to be transferred before execution.
	/// * `message` - Message to be passed to the entrypoint.
	/// * `event_handler` - Event handler that is passed by the VM.
	pub(crate) fn continue_run(
		self,
		shared: &mut CosmwasmVMShared,
		funds: Vec<Coin>,
		message: &[u8],
		event_handler: &mut dyn FnMut(cosmwasm_vm::cosmwasm_std::Event),
	) -> Result<Option<cosmwasm_vm::cosmwasm_std::Binary>, CosmwasmVMError<T>>
	where
		for<'x> WasmiVM<DefaultCosmwasmVM<'x, T>>:
			CosmwasmCallVM<I> + CosmwasmDynamicVM<I> + StargateCosmwasmCallVM,
		for<'x> VmErrorOf<WasmiVM<DefaultCosmwasmVM<'x, T>>>:
			From<CosmwasmVMError<T>> + Into<CosmwasmVMError<T>>,
	{
		// Call `cosmwasm_call` to transfer funds and create the vm instance before
		// calling the callback.
		Pallet::<T>::cosmwasm_call(
			shared,
			self.sender,
			self.contract,
			funds,
			// `cosmwasm_system_run` is called instead of `cosmwasm_system_entrypoint` here
			// because here, we want to continue running the transaction with the given
			// entrypoint
			|vm| {
				cosmwasm_system_run_hook::<I, _>(vm, message, event_handler, |vm, message| match vm
					.0
					.contract_runtime
				{
					ContractBackend::CosmWasm { .. } =>
						cosmwasm_call::<I, _>(vm, message).map(Into::into),
					ContractBackend::Pallet => T::PalletHook::execute(vm, self.entrypoint, message),
				})
				.map_err(Into::into)
			},
		)
	}
}
