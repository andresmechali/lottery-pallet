#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{pallet_prelude::MaxEncodedLen, traits::Currency, RuntimeDebug};
// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(
	Encode, Decode, Default, Eq, PartialEq, scale_info::TypeInfo, MaxEncodedLen, RuntimeDebug, Clone,
)]
pub enum RouletteType {
	Color,
	Dozen,
	#[default]
	Full,
}

#[derive(
	Encode, Decode, Default, Eq, PartialEq, scale_info::TypeInfo, MaxEncodedLen, RuntimeDebug,
)]
pub struct TicketData<AccountId, BlockNumber, Balance> {
	/// Ticket id.
	id: u64,
	/// Ticket buyer.
	owner: AccountId,
	/// Bet amount.
	amount: Balance,
	/// Block in which bet occurs.
	block: BlockNumber,
	/// Type of bet.
	roulette_type: RouletteType,
}

#[frame_support::pallet]
pub mod pallet {
	use crate::{BalanceOf, RouletteType, TicketData};
	use frame_support::{
		pallet_prelude::*,
		traits::{Randomness},
		PalletId
	};
	use frame_support::traits::{Currency, ExistenceRequirement};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;
	use sp_runtime::{traits::AccountIdConversion, Saturating};

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type LotteryRandomness: Randomness<H256, u32>;

		// type Currency: ReservableCurrency<Self::AccountId>;
		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	// Pallets use events to inform users when important changes are made.
	// Event documentation should end with an array that provides descriptive names for parameters.
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// TODO: create generic type for TicketId and Prize
		/// Event emitted when a ticket has been issued.
		TicketIssued { who: T::AccountId, ticket_id: u64 },
		/// Event emitted when a prized has been paid.
		PrizePaid { who: T::AccountId, ticket_id: u64, prize: u64 },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Not enough participants.
		NotEnoughParticipants,
		/// Not enough balance to afford a ticket.
		NotEnoughBalance,
	}

	#[pallet::type_value]
	pub fn DefaultTicketNonce<T: Config>() -> u64 {
		0u64
	}

	#[pallet::storage]
	pub(super) type TicketNonce<T: Config> =
		StorageValue<_, u64, ValueQuery, DefaultTicketNonce<T>>;

	#[pallet::storage]
	pub(super) type Tickets<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		u64,
		TicketData<T::AccountId, T::BlockNumber, BalanceOf<T>>,
	>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn buy_ticket(
			origin: OriginFor<T>,
			amount: BalanceOf<T>,
			roulette_type: RouletteType,
		) -> DispatchResult {
			// TODO: allow to buy multiple tickets, each for a fixed amount of tokens

			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let sender = ensure_signed(origin)?;

			// Verify that the buyer has enough balance to afford the ticket and is
			// left with more than the existential deposit
			let total_balance = T::Currency::total_balance(&sender);
			let existential_deposit = T::Currency::minimum_balance();
			ensure!(total_balance.saturating_sub(amount) >= existential_deposit, Error::<T>::NotEnoughBalance);

			// Verify that the specified claim has not already been stored.
			// ensure!(!Claims::<T>::contains_key(&claim), Error::<T>::AlreadyClaimed);

			// Get the block number from the FRAME System pallet.
			let current_block = <frame_system::Pallet<T>>::block_number();

			let ticket_id = Self::get_and_increment_nonce();

			let account_id = Self::account_id();

			// TODO: get random roulette number

			// Transfer balance
			T::Currency::transfer(&sender, &account_id, amount, ExistenceRequirement::KeepAlive)?;

			// Store the ticket ownership.
			Tickets::<T>::insert(
				ticket_id,
				TicketData {
					id: ticket_id,
					owner: sender.clone(),
					amount,
					block: current_block,
					roulette_type,
				},
			);

			// Emit an event showing that the claim was created.
			Self::deposit_event(Event::TicketIssued { who: sender, ticket_id });

			Ok(())
		}
	}

	// Helper functions
	impl<T: Config> Pallet<T> {
		fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		fn get_and_increment_nonce() -> u64 {
			// Note: Can this be atomic to avoid a race?
			let nonce = TicketNonce::<T>::get();
			TicketNonce::<T>::put(nonce.wrapping_add(1));
			nonce
		}
	}
}
