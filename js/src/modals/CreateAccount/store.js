// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import { action, observable } from 'mobx';

const STAGE_SELECT_TYPE = 0;

export default class Store {
  @observable createType = 'fromNew';
  @observable stage = STAGE_SELECT_TYPE;

  constructor (api) {
    this._api = api;
  }

  @action setCreateType = (createType) => {
    this.createType = createType;
  }

  @action setStage = (stage) => {
    this.stage = stage;
  }

  @action nextStage = () => {
    this.stage++;
  }

  @action prevStage = () => {
    this.stage--;
  }
}

export {
  STAGE_SELECT_TYPE
};
