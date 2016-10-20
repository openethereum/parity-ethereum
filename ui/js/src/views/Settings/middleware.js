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

import defaultViews from './Views/defaults';

function initBackground (store, api) {
  const backgroundSeed = loadBackground() || api.util.sha3(`${Date.now()}`);

  store.dispatch({
    type: 'updateBackground',
    backgroundSeed
  });
}

function loadBackground () {
  return window.localStorage.getItem('backgroundSeed');
}

function saveBackground (backgroundSeed) {
  window.localStorage.setItem('backgroundSeed', backgroundSeed);
}

function initViews (store) {
  const { settings } = store.getState();
  const data = loadViews();
  const viewIds = Object.keys(data).filter((viewId) => {
    return settings.views[viewId] && data[viewId].active !== settings.views[viewId].active;
  });

  if (viewIds.length) {
    store.dispatch({ type: 'toggleViews', viewIds });
  }
}

function getFixedViews () {
  const views = {};

  Object.keys(defaultViews).forEach((id) => {
    if (defaultViews[id].fixed) {
      views[id] = { active: true };
    }
  });

  return views;
}

function getDefaultViews () {
  const views = {};

  Object.keys(defaultViews).forEach((id) => {
    views[id] = {
      active: defaultViews[id].active || false
    };
  });

  return views;
}

function loadViews () {
  const fixed = getFixedViews();
  const defaults = getDefaultViews();
  let data;

  try {
    const json = window.localStorage.getItem('views') || '{}';

    data = Object.assign(defaults, JSON.parse(json), fixed);
  } catch (e) {
    data = defaults;
  }

  return data;
}

function saveViews (store) {
  window.localStorage.setItem('views', JSON.stringify(getDefaultViews()));
}

function toggleViews (store, viewIds) {
  viewIds.forEach((id) => {
    defaultViews[id].active = !defaultViews[id].active;
  });

  saveViews(store);
}

export default class SettingsMiddleware {
  toMiddleware () {
    return (store) => (next) => (action) => {
      switch (action.type) {
        case 'initAll':
          initBackground(store, action.api);
          initViews(store);
          break;

        case 'toggleView':
          toggleViews(store, [action.viewId]);
          break;

        case 'toggleViews':
          toggleViews(store, action.viewIds);
          break;

        case 'updateBackground':
          saveBackground(action.backgroundSeed);
          break;
      }

      next(action);
    };
  }
}
