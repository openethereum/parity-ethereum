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

const initialState = {};

export default (state = initialState, action) => {
  if (action.type === 'addCertification') {
    const { address, id, name, icon, title } = action;
    const certifications = state[address] || [];

    if (certifications.some((c) => c.id === id)) {
      return state;
    }

    const newCertifications = certifications.concat({
      id, name, icon, title
    });

    return { ...state, [address]: newCertifications };
  }

  if (action.type === 'removeCertification') {
    const { address, id } = action;
    const certifications = state[address] || [];

    const newCertifications = certifications.filter((c) => c.id !== id);

    return { ...state, [address]: newCertifications };
  }

  return state;
};
