import LocalStorage from './localstorage';

export default function () {
  const localstorage = new LocalStorage();

  return [
    localstorage.toMiddleware()
  ];
}
