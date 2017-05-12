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

import BigNumber from 'bignumber.js';

const WEI_TO_ETH_MULTIPLIER = 0.000000000000000001;
const WEI_TO_SZABU_MULTIPLIER = 0.000000000001;

export const getShortData = _getShortData;
// calculations
export const getFee = _getFee;
export const calcFeeInEth = _calcFeeInEth;
export const getTotalValue = _getTotalValue;
// displays
export const getSzaboFromWeiDisplay = _getSzaboFromWeiDisplay;
export const getValueDisplay = _getValueDisplay;
export const getValueDisplayWei = _getValueDisplayWei;
export const getTotalValueDisplay = _getTotalValueDisplay;
export const getTotalValueDisplayWei = _getTotalValueDisplayWei;
export const getEthmFromWeiDisplay = _getEthmFromWeiDisplay;
export const getGasDisplay = _getGasDisplay;

function _getShortData (data) {
  if (data.length <= 3) {
    return data;
  }
  return data.substr(0, 3) + '...';
}

/*
 * @param {hex string} gas
 * @param {wei hex string} gasPrice
 * @return {BigNumber} fee in wei
 */
function _getFee (gas, gasPrice) {
  gas = new BigNumber(gas);
  gasPrice = new BigNumber(gasPrice);
  return gasPrice.times(gas);
}

function _calcFeeInEth (totalValue, value) {
  let fee = new BigNumber(totalValue).sub(new BigNumber(value));

  return fee.times(WEI_TO_ETH_MULTIPLIER).toFormat(7);
}

/*
 * @param {wei BigNumber} fee
 * @param {wei hex string} value
 * @return {BigNumber} total value in wei
 */
function _getTotalValue (fee, value) {
  value = new BigNumber(value);
  return fee.plus(value);
}

/*
 * @param {wei hex string} gasPrice
 * @return {string} szabo gas price with unit [szabo] i.e. 21,423 [szabo]
 */
function _getSzaboFromWeiDisplay (gasPrice) {
  gasPrice = new BigNumber(gasPrice);
  return gasPrice.times(WEI_TO_SZABU_MULTIPLIER).toPrecision(5);
}

/*
 * @param {wei hex string} value
 * @return {string} value in WEI nicely formatted
 */
function _getValueDisplay (value) {
  value = new BigNumber(value);
  return value.times(WEI_TO_ETH_MULTIPLIER).toFormat(5);
}

function _getValueDisplayWei (value) {
  value = new BigNumber(value);
  return value.toFormat(0);
}

/*
 * @param {wei hex string} totalValue
 * @return {string} total value (including fee) with units i.e. 1.32 [eth]
 */
function _getTotalValueDisplay (totalValue) {
  totalValue = new BigNumber(totalValue);
  return totalValue.times(WEI_TO_ETH_MULTIPLIER).toFormat(5);
}

function _getTotalValueDisplayWei (totalValue) {
  totalValue = new BigNumber(totalValue);
  return totalValue.toFormat(0);
}

function _getEthmFromWeiDisplay (weiHexString) {
  const value = new BigNumber(weiHexString);

  return value.times(WEI_TO_ETH_MULTIPLIER).times(1e7).toFixed(5);
}

function _getGasDisplay (gas) {
  return new BigNumber(gas).times(1e-7).toFormat(4);
}
