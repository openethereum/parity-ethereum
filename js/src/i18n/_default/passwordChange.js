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

export default {
  title: `Password Manager`,
  tabTest: {
    label: `Test Password`
  },
  testPassword: {
    hint: `your account password`,
    label: `password`
  },
  tabChange: {
    label: `Change Password`
  },
  currentPassword: {
    hint: `your current password for this account`,
    label: `current password`
  },
  passwordHint: {
    hint: `hint for the new password`,
    label: `(optional) new password hint`
  },
  newPassword: {
    hint: `the new password for this account`,
    label: `new password`
  },
  repeatPassword: {
    error: `the supplied passwords do not match`,
    hint: `repeat the new password for this account`,
    label: `repeat new password`
  },
  button: {
    cancel: `Cancel`,
    wait: `Wait...`,
    test: `Test`,
    change: `Change`
  },
  success: `Your password has been successfully changed`
};
