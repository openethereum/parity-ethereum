// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Auto-updates minimal gas price requirement.

use std::time::{Instant, Duration};

use ansi_term::Colour;
use ethereum_types::U256;
use price_info::{Client as PriceInfoClient, PriceInfo};
use price_info::fetch::Client as FetchClient;

/// Options for the dynamic gas price recalibrator.
#[derive(Debug, PartialEq)]
pub struct GasPriceCalibratorOptions {
	/// Base transaction price to match against.
	pub usd_per_tx: f32,
	/// How frequently we should recalibrate.
	pub recalibration_period: Duration,
}

/// The gas price validator variant for a `GasPricer`.
#[derive(Debug, PartialEq)]
pub struct GasPriceCalibrator {
	options: GasPriceCalibratorOptions,
	next_calibration: Instant,
	price_info: PriceInfoClient,
}

impl GasPriceCalibrator {
	fn recalibrate<F: Fn(U256) + Sync + Send + 'static>(&mut self, set_price: F) {
		trace!(target: "miner", "Recalibrating {:?} versus {:?}", Instant::now(), self.next_calibration);
		if Instant::now() >= self.next_calibration {
			let usd_per_tx = self.options.usd_per_tx;
			trace!(target: "miner", "Getting price info");

			self.price_info.get(move |price: PriceInfo| {
				trace!(target: "miner", "Price info arrived: {:?}", price);
				let usd_per_eth = price.ethusd;
				let wei_per_usd: f32 = 1.0e18 / usd_per_eth;
				let gas_per_tx: f32 = 21000.0;
				let wei_per_gas: f32 = wei_per_usd * usd_per_tx / gas_per_tx;
				info!(target: "miner", "Updated conversion rate to Ξ1 = {} ({} wei/gas)", Colour::White.bold().paint(format!("US${:.2}", usd_per_eth)), Colour::Yellow.bold().paint(format!("{}", wei_per_gas)));
				set_price(U256::from(wei_per_gas as u64));
			});

			self.next_calibration = Instant::now() + self.options.recalibration_period;
		}
	}
}

/// Struct to look after updating the acceptable gas price of a miner.
#[derive(Debug, PartialEq)]
pub enum GasPricer {
	/// A fixed gas price in terms of Wei - always the argument given.
	Fixed(U256),
	/// Gas price is calibrated according to a fixed amount of USD.
	Calibrated(GasPriceCalibrator),
}

impl GasPricer {
	/// Create a new Calibrated `GasPricer`.
	pub fn new_calibrated(options: GasPriceCalibratorOptions, fetch: FetchClient) -> GasPricer {
		GasPricer::Calibrated(GasPriceCalibrator {
			options: options,
			next_calibration: Instant::now(),
			price_info: PriceInfoClient::new(fetch),
		})
	}

	/// Create a new Fixed `GasPricer`.
	pub fn new_fixed(gas_price: U256) -> GasPricer {
		GasPricer::Fixed(gas_price)
	}

	/// Recalibrate current gas price.
	pub fn recalibrate<F: Fn(U256) + Sync + Send + 'static>(&mut self, set_price: F) {
		match *self {
			GasPricer::Fixed(ref max) => set_price(max.clone()),
			GasPricer::Calibrated(ref mut cal) => cal.recalibrate(set_price),
		}
	}
}
