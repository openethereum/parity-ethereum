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

import { handleActions } from 'redux-actions';

import views from './Views/defaults';

const backgroundSeed = window.localStorage.getItem('backgroundSeed') || `${Date.now()}`;
window.localStorage.setItem('backgroundSeed', backgroundSeed);

const initialState = {
  views,
  backgroundSeed
};

export default handleActions({
  toggleView (state, action) {
    const { viewId } = action;

    state.views[viewId].active = !state.views[viewId].active;

    return Object.assign({}, state);
  },

  updateBackground (state, action) {
    const { backgroundSeed } = action;

    return Object.assign({}, state, { backgroundSeed });
  }
}, initialState);
