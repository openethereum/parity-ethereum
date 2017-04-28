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

export const onPrint = (window, cb) => {
  let called = false;
  let query;
  let queryFn;

  const onPrint = () => {
    if (queryFn) {
      query.removeListener(queryFn);
    }

    window.removeEventListener('afterprint', onPrint, false);

    if (!called) {
      called = true;
      cb();
    }
  };

  if (window.matchMedia) {
    queryFn = (query) => {
      if (!query.matches) {
        onPrint();
      }
    };

    query = window.matchMedia('print');
    query.addListener(queryFn);
  }

  window.addEventListener('afterprint', onPrint, false);
};

export default (html) => {
  const iframe = document.createElement('iframe');

  iframe.setAttribute('sandbox', 'allow-modals allow-same-origin allow-scripts');
  iframe.setAttribute('src', '/');
  iframe.setAttribute('style', 'display: none');
  document.body.appendChild(iframe);
  const teardown = () => {
    // Safari crashes without a timeout.
    setTimeout(() => document.body.removeChild(iframe), 0);
  };

  setTimeout(() => {
    iframe.contentDocument.write(html);

    setTimeout(() => {
      onPrint(iframe.contentWindow, teardown);
      iframe.contentWindow.focus();
      iframe.contentWindow.print();
    }, 20);
  }, 0);
};
