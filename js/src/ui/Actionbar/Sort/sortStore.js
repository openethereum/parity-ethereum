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

import { action, observable } from 'mobx';
import store from 'store';

const LS_STORE_KEY = '_parity::sortStore';

export default class SortStore {
  @observable menuOpen = false;

  constructor (props) {
    const { id, onChange } = props;

    this.onChange = onChange;
    this.id = id;
  }

  @action handleMenuOpen = () => {
    this.menuOpen = true;
  }

  @action handleMenuChange = (open) => {
    this.menuOpen = open;
  }

  @action handleSortChange = (event, child) => {
    const order = child.props.value;

    this.onChange(order);
    this.saveOrder(order);
  }

  @action restoreSavedOrder = () => {
    const order = this.getSavedOrder();

    this.onChange(order);
  }

  getSavedOrder = () => {
    return (this.getSavedOrders())[this.id];
  }

  getSavedOrders = () => {
    return store.get(LS_STORE_KEY) || {};
  }

  setSavedOrders = (orders) => {
    store.set(LS_STORE_KEY, orders);
  }

  saveOrder = (order) => {
    const orders = {
      ...this.getSavedOrders(),
      [ this.id ]: order
    };

    this.setSavedOrders(orders);
  }
}
