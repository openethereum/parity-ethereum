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

export function scrollTo (element, to, duration) {
  let start = element.scrollTop;
  let change = to - start;
  let increment = 50;

  let animateScroll = elapsedTime => {
    elapsedTime += increment;
    let position = easeInOut(elapsedTime, start, change, duration);

    element.scrollTop = position;
    if (elapsedTime < duration) {
      setTimeout(() => {
        // stop if user scrolled
        if (element.scrollTop !== parseInt(position, 10)) {
          return;
        }
        animateScroll(elapsedTime);
      }, increment);
    }
  };

  animateScroll(0);
}

export function easeInOut (currentTime, start, change, duration) {
  currentTime /= duration / 2;
  if (currentTime < 1) {
    return change / 2 * currentTime * currentTime + start;
  }
  currentTime -= 1;
  return -change / 2 * (currentTime * (currentTime - 2) - 1) + start;
}
