#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{pallet_prelude::MaxEncodedLen, traits::Currency, RuntimeDebug};
// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(
	Encode, Decode, Eq, PartialEq, scale_info::TypeInfo, MaxEncodedLen, RuntimeDebug, Clone,
)]
pub enum DozenOrColumn {
	First,
	Second,
	Third,
}

#[derive(
	Encode, Decode, Eq, PartialEq, scale_info::TypeInfo, MaxEncodedLen, RuntimeDebug, Clone,
)]
pub enum Bet {
	ColorPick(RouletteColor),
	FullPick(u32),
	DozenPick(DozenOrColumn),
	ColumnPick(DozenOrColumn),
}

#[derive(Encode, Decode, Eq, PartialEq, scale_info::TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct BetData<AccountId, BlockNumber, Balance> {
	/// Bet id.
	id: u64,
	/// Owner of the bet.
	owner: AccountId,
	/// Bet amount.
	amount: Balance,
	/// Block in which bet occurs.
	block: BlockNumber,
	/// Winner number.
	winner_number: u32,
	/// Type of bet.
	bet: Bet,
}

#[derive(
	Encode, Decode, Eq, PartialEq, scale_info::TypeInfo, MaxEncodedLen, RuntimeDebug, Clone,
)]
pub enum RouletteColor {
	Red,
	Black,
	Green,
}

trait GetColor {
	fn to_color(&self) -> Option<RouletteColor>;
}

impl GetColor for u32 {
	fn to_color(&self) -> Option<RouletteColor> {
		match self {
			0 => Some(RouletteColor::Green),
			1..=10 | 19..=28 => {
				if self % 2 == 0 {
					Some(RouletteColor::Black)
				} else {
					Some(RouletteColor::Red)
				}
			},
			11..=18 | 29..=36 => {
				if self % 2 == 0 {
					Some(RouletteColor::Red)
				} else {
					Some(RouletteColor::Black)
				}
			},
			_ => None,
		}
	}
}

#[frame_support::pallet]
pub mod pallet {
	use crate::{BalanceOf, Bet, BetData, DozenOrColumn, GetColor, RouletteColor};
	use frame_support::traits::{Currency, ExistenceRequirement};
	use frame_support::{pallet_prelude::*, traits::Randomness, PalletId};
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
		// TODO: create generic type for BetId and Prize
		/// Event emitted when a bet has been placed
		BetPlaced { bet_id: u64, who: T::AccountId, bet: Bet, amount: BalanceOf<T> },
		/// Event emitted when a game was won.
		RouletteWon {
			who: T::AccountId,
			bet_id: u64,
			bet: Bet,
			winner_number: u32,
			winner_color: Option<RouletteColor>,
			prize: BalanceOf<T>,
		},
		/// Event emitted when a game was lost.
		RouletteLost {
			who: T::AccountId,
			bet_id: u64,
			bet: Bet,
			winner_number: u32,
			winner_color: Option<RouletteColor>,
			amount: BalanceOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Not enough balance to afford the bet.
		NotEnoughBalance,
		/// Number must be between 0 and 36
		OutOfRange,
	}

	#[pallet::type_value]
	pub fn DefaultBetNonce<T: Config>() -> u64 {
		0u64
	}

	#[pallet::storage]
	pub(super) type BetNonce<T: Config> = StorageValue<_, u64, ValueQuery, DefaultBetNonce<T>>;

	#[pallet::storage]
	pub(super) type Bets<T: Config> =
		StorageMap<_, Blake2_128Concat, u64, BetData<T::AccountId, T::BlockNumber, BalanceOf<T>>>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn place_bet(origin: OriginFor<T>, amount: BalanceOf<T>, bet: Bet) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			let sender = ensure_signed(origin)?;

			// Verify that the buyer has enough balance to afford the bet and is
			// left with more than the existential deposit.
			let total_balance = T::Currency::total_balance(&sender);
			let existential_deposit = T::Currency::minimum_balance();
			ensure!(
				total_balance.saturating_sub(amount) >= existential_deposit,
				Error::<T>::NotEnoughBalance
			);

			// TODO: Verify that the pallet account has enough balance to afford a payout
			// For this we need to keep track of all the possible payouts of the current block.

			// Get the block number.
			let current_block = <frame_system::Pallet<T>>::block_number();

			// Get random roulette number
			let random_number = Self::random_number(37_u32);

			// Generate new bet
			let bet_id = Self::get_and_increment_nonce();
			let account_id = Self::account_id();

			// Store the bet.
			Bets::<T>::insert(
				bet_id,
				BetData {
					id: bet_id,
					owner: sender.clone(),
					amount,
					block: current_block,
					winner_number: random_number,
					bet: bet.clone(),
				},
			);

			// Emit an event showing that the claim was created.
			Self::deposit_event(Event::BetPlaced {
				who: sender.clone(),
				bet_id,
				amount,
				bet: bet.clone(),
			});

			let is_winner = Self::is_winner(bet.clone(), random_number);

			if is_winner {
				let payout_amount = Self::amount_won(bet.clone(), amount);
				// Transfer balance
				T::Currency::transfer(
					&account_id,
					&sender,
					payout_amount,
					ExistenceRequirement::KeepAlive,
				)?;

				Self::deposit_event(Event::RouletteWon {
					who: sender,
					bet_id,
					bet,
					winner_number: random_number,
					winner_color: random_number.to_color(),
					prize: payout_amount,
				});
			} else {
				T::Currency::transfer(
					&sender,
					&account_id,
					amount,
					ExistenceRequirement::KeepAlive,
				)?;

				Self::deposit_event(Event::RouletteLost {
					who: sender,
					bet_id,
					bet,
					winner_number: random_number,
					winner_color: random_number.to_color(),
					amount,
				});
			}

			Ok(())
		}
	}

	// Helper functions
	impl<T: Config> Pallet<T> {
		fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		fn get_and_increment_nonce() -> u64 {
			let nonce = BetNonce::<T>::get();
			BetNonce::<T>::put(nonce.wrapping_add(1));
			nonce
		}

		fn random_number(total: u32) -> u32 {
			let (random_seed, _) = T::LotteryRandomness::random_seed();
			let random_number = <u32>::decode(&mut random_seed.as_ref())
				.expect("secure hashes should always be bigger than u32; qed");
			random_number % total
		}

		fn is_color_winner(color: RouletteColor, winner_number: u32) -> bool {
			match winner_number.to_color() {
				Some(winner_color) => winner_color == color,
				None => false,
			}
		}

		fn is_dozen_winner(dozen: DozenOrColumn, winner_number: u32) -> bool {
			match winner_number {
				0 => false,
				1..=12 => dozen == DozenOrColumn::First,
				13..=24 => dozen == DozenOrColumn::Second,
				25..=36 => dozen == DozenOrColumn::Third,
				_ => false,
			}
		}

		// TODO: change conditions
		fn is_column_winner(column: DozenOrColumn, winner_number: u32) -> bool {
			match winner_number {
				0 => false,
				1..=12 => column == DozenOrColumn::First,
				13..=24 => column == DozenOrColumn::Second,
				25..=36 => column == DozenOrColumn::Third,
				_ => false,
			}
		}

		fn is_full_winner(picked_number: u32, winner_number: u32) -> bool {
			picked_number == winner_number
		}

		fn is_winner(pick: Bet, winner_number: u32) -> bool {
			match pick {
				Bet::ColorPick(color) => Self::is_color_winner(color, winner_number),
				Bet::FullPick(number) => Self::is_full_winner(number, winner_number),
				Bet::DozenPick(dozen) => Self::is_dozen_winner(dozen, winner_number),
				Bet::ColumnPick(column) => Self::is_column_winner(column, winner_number),
			}
		}

		fn amount_won(pick: Bet, amount: BalanceOf<T>) -> BalanceOf<T> {
			match pick {
				// TODO: use consts
				Bet::ColorPick(_) => amount.saturating_mul(BalanceOf::<T>::from(2_u32)),
				Bet::FullPick(_) => amount.saturating_mul(BalanceOf::<T>::from(36_u32)),
				Bet::DozenPick(_) => amount.saturating_mul(BalanceOf::<T>::from(3_u32)),
				Bet::ColumnPick(_) => amount.saturating_mul(BalanceOf::<T>::from(3_u32)),
			}
		}
	}
}
