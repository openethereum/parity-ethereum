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

export function createLocation (token, location = window.location) {
  const { hash, port, protocol } = location;
  let query = '';

  if (hash && hash.indexOf('?') !== -1) {
    // TODO: currently no app uses query-params visible in the shell, this may need adjustment if they do
    query = hash;
  } else {
    query = `${hash || '#/'}${token ? '?token=' : ''}${token || ''}`;
  }

  return `${protocol}//127.0.0.1:${port}/${query}`;
}

export function redirectLocalhost (token) {
  // we don't want localhost, rather we want 127.0.0.1
  if (window.location.hostname !== 'localhost') {
    return false;
  }

  window.location.assign(createLocation(token, window.location));

  return true;
}
