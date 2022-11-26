#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{inherent::Vec, pallet_prelude::MaxEncodedLen, RuntimeDebug};

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
pub enum Half {
	First,
	Second,
}

#[derive(
	Encode, Decode, Eq, PartialEq, scale_info::TypeInfo, MaxEncodedLen, RuntimeDebug, Clone,
)]
pub enum OddOrEven {
	Odd,
	Even,
}

#[derive(
	Encode, Decode, Eq, PartialEq, scale_info::TypeInfo, MaxEncodedLen, RuntimeDebug, Clone,
)]
pub enum Bet {
	Color(RouletteColor),
	Full(u32),
	Dozen(DozenOrColumn),
	Column(DozenOrColumn),
	Half(Half),
	OddOrEven(OddOrEven),
}

#[derive(Encode, Decode, Eq, PartialEq, scale_info::TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct BetData<AccountId, BlockNumber, Balance> {
	/// Bet id.
	pub id: u64,
	/// Owner of the bet.
	pub owner: AccountId,
	/// Bet amount.
	pub amount: Balance,
	/// Block in which bet occurs.
	pub block: BlockNumber,
	/// Type of bet.
	pub bet: Bet,
}

#[derive(Encode, Decode, Eq, PartialEq, scale_info::TypeInfo, MaxEncodedLen, RuntimeDebug)]
pub struct RouletteResult<AccountId, BlockNumber, Balance> {
	/// Block in which game took place.
	pub block: BlockNumber,
	/// Bets that participated in roulette.
	pub bets: Vec<BetData<AccountId, BlockNumber, Balance>>,
	/// Winner number.
	pub winner_number: u32,
	/// Amount received from losers.
	pub income: Balance,
	/// Amount paid to winners.
	pub payout: Balance,
}

#[derive(
	Encode, Decode, Eq, PartialEq, scale_info::TypeInfo, MaxEncodedLen, RuntimeDebug, Clone,
)]
pub enum RouletteColor {
	Red,
	Black,
	Green,
}

pub trait RouletteNumber {
	fn to_color(&self) -> Option<RouletteColor>;

	fn to_dozen(&self) -> Option<DozenOrColumn>;

	fn to_column(&self) -> Option<DozenOrColumn>;

	fn is_even(&self) -> bool;

	fn to_half(&self) -> Option<Half>;
}

impl RouletteNumber for u32 {
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

	fn to_dozen(&self) -> Option<DozenOrColumn> {
		match self {
			1..=12 => Some(DozenOrColumn::First),
			13..=24 => Some(DozenOrColumn::Second),
			25..=36 => Some(DozenOrColumn::Third),
			_ => None,
		}
	}

	fn to_column(&self) -> Option<DozenOrColumn> {
		match self {
			1..=36 => {
				if self % 3 == 1 {
					Some(DozenOrColumn::First)
				} else if self % 3 == 2 {
					Some(DozenOrColumn::Second)
				} else {
					Some(DozenOrColumn::Third)
				}
			},
			_ => None,
		}
	}

	fn is_even(&self) -> bool {
		match self {
			1..=36 => self % 2 == 0,
			_ => false,
		}
	}

	fn to_half(&self) -> Option<Half> {
		match self {
			1..=18 => Some(Half::First),
			19..=36 => Some(Half::Second),
			_ => None,
		}
	}
}
