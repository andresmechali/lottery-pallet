#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{pallet_prelude::*, traits::Randomness};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type LotteryRandomness: Randomness<H256, u32>;
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
	pub(super) type Tickets<T: Config> = StorageMap<_, Blake2_128Concat, u64, T::AccountId>; // ticket id -> account id

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn buy_ticket(origin: OriginFor<T>) -> DispatchResult {
			// TODO: allow to buy multiple tickets, each for a fixed amount of tokens

			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let sender = ensure_signed(origin)?;

			// TODO: Verify that the buyer has enough balance to afford the ticket

			// TODO: Transfer balance

			// Verify that the specified claim has not already been stored.
			// ensure!(!Claims::<T>::contains_key(&claim), Error::<T>::AlreadyClaimed);

			// Get the block number from the FRAME System pallet.
			// let current_block = <frame_system::Pallet<T>>::block_number();

			let ticket_id = Self::get_and_increment_nonce();

			// Store the ticket ownership
			Tickets::<T>::insert(ticket_id, &sender);

			// Emit an event that the claim was created.
			Self::deposit_event(Event::TicketIssued { who: sender, ticket_id });

			Ok(())
		}
	}

	// Helper functions
	impl<T: Config> Pallet<T> {
		fn get_and_increment_nonce() -> u64 {
			// Note: Can this be atomic to avoid a race?
			let nonce = TicketNonce::<T>::get();
			TicketNonce::<T>::put(nonce.wrapping_add(1));
			nonce
		}
	}
}
