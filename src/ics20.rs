///
/// ics20 transfer keeper
/// refer to https://github.com/octopus-network/ibc-go/blob/main/modules/apps/transfer/keeper/keeper.go
use super::*;
use crate::routing::Context;
use ibc::{
	applications::ics20_fungible_token_transfer::{
		context::{AccountReader, BankKeeper, BankReader, Ics20Context, Ics20Keeper, Ics20Reader},
		error::Error as Ics20Error,
		DenomTrace, HashedDenom, IbcCoin,
	},
	core::ics24_host::identifier::PortId,
};

impl<T: Config> BankReader for Context<T> {
	type AccountId = AccountId<T>;

	/// Returns true if the specified account is not allowed to receive funds and false otherwise.
	fn is_blocked_account(&self, account: &Self::AccountId) -> bool {
		todo!()
	}

	/// get_transfer_account returns the ICS20 - transfers AccountId.
	fn get_transfer_account(&self) -> Self::AccountId {
		todo!()
	}
}

impl<T: Config> BankKeeper for Context<T> {
	type AccountId = AccountId<T>;

	/// This function should enable sending ibc fungible tokens from one account to another
	fn send_coins(
		&mut self,
		from: &Self::AccountId,
		to: &Self::AccountId,
		amt: &IbcCoin,
	) -> Result<(), Ics20Error> {
		todo!()
	}

	/// This function to enable minting ibc tokens in a module
	fn mint_coins(&mut self, module: &Self::AccountId, amt: &IbcCoin) -> Result<(), Ics20Error> {
		todo!()
	}

	/// This function should enable burning of minted tokens
	fn burn_coins(&mut self, module: &Self::AccountId, amt: &IbcCoin) -> Result<(), Ics20Error> {
		todo!()
	}

	/// This function should enable transfer of tokens from the ibc module to an account
	fn send_coins_from_module_to_account(
		&mut self,
		module: &Self::AccountId,
		to: &Self::AccountId,
		amt: &IbcCoin,
	) -> Result<(), Ics20Error> {
		todo!()
	}

	/// This function should enable transfer of tokens from an account to the ibc module
	fn send_coins_from_account_to_module(
		&mut self,
		from: &Self::AccountId,
		module: &Self::AccountId,
		amt: &IbcCoin,
	) -> Result<(), Ics20Error> {
		todo!()
	}
}

impl<T: Config> AccountReader for Context<T> {
	type AccountId = AccountId<T>;
	type Address = AccountId<T>; // todo

	fn get_account(&self, address: &Self::Address) -> Option<Self::AccountId> {
		todo!()
	}
}

impl<T: Config> Ics20Reader for Context<T> {
	type AccountId = AccountId<T>;

	/// get_port returns the portID for the transfer module.
	fn get_port(&self) -> Result<PortId, Ics20Error> {
		todo!()
	}

	/// Returns true iff send is enabled.
	fn is_send_enabled(&self) -> bool {
		todo!()
	}

	/// Returns true iff receive is enabled.
	fn is_receive_enabled(&self) -> bool {
		todo!()
	}

	/// Get the denom trace associated with the specified hash in the store.
	fn get_denom_trace(&self, denom_hash: &HashedDenom) -> Option<DenomTrace> {
		todo!()
	}
}

impl<T: Config> Ics20Keeper for Context<T> {
	type AccountId = AccountId<T>;

	/// Sets a new {trace hash -> denom trace} pair to the store.
	fn set_denom_trace(&mut self, denom_trace: &DenomTrace) -> Result<(), Ics20Error> {
		todo!()
	}
}

impl<T: Config> Ics20Context for Context<T> {
	type AccountId = AccountId<T>;
}

pub struct AccountId<T: Config>(T::AccountId);

impl<T: Config> From<AccountId<T>> for alloc::string::String {
	fn from(account_id: AccountId<T>) -> Self {
		todo!()
	}
}

impl<T: Config> FromStr for AccountId<T> {
	type Err = Ics20Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		todo!()
	}
}
