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

import Push from 'push.js';
import BigNumber from 'bignumber.js';

import unknownIcon from '~/../assets/images/contracts/unknown-64x64.png';

export function notifyTransaction (account, token, _value, onClick) {
  const name = account.name || account.address;
  const value = _value.div(new BigNumber(token.format || 1));
  const icon = token.image || unknownIcon;

  let _notification = null;

  Push
    .create(`${name}`, {
      body: `You just received ${value.toFormat(3)} ${token.tag.toUpperCase()}`,
      icon: {
        x16: icon,
        x32: icon
      },
      timeout: 20000,
      onClick: () => {
        // Focus on the UI
        try {
          window.focus();
        } catch (e) {}

        if (onClick && typeof onClick === 'function') {
          onClick();
        }

        // Close the notification
        if (_notification) {
          _notification.close();
          _notification = null;
        }
      }
    })
    .then((notification) => {
      _notification = notification;
    });
}
