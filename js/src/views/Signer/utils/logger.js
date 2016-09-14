// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

// this module disable logging on prod

const isLogging = process.env.LOGGING;

export default logger();

function logger () {
  return !isLogging ? prodLogger() : devLogger();
}

function prodLogger () {
  return {
    log: noop,
    info: noop,
    error: noop,
    warn: noop
  };
}

function devLogger () {
  return {
    log: console.log.bind(console),
    info: console.info.bind(console),
    error: console.error.bind(console),
    warn: console.warn.bind(console)
  };
}

function noop () {}
