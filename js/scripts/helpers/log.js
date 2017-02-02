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

import chalk from 'chalk';

// INFO Logging helper
export function info (log) {
  console.log(chalk.blue(`INFO:\t${log}`));
}

// WARN Logging helper
export function warn (log) {
  console.warn(chalk.yellow(`WARN:\t${log}`));
}

// ERROR Logging helper
export function error (log) {
  console.error(chalk.red(`ERROR:\t${log}`));
}
