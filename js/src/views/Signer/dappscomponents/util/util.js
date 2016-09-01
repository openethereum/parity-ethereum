
export function toPromise (fn) {
  return new Promise((resolve, reject) => {
    fn((err, res) => {
      if (err) {
        reject(err);
      } else {
        resolve(res);
      }
    });
  });
}

export function identity (x) {
  return x;
}

export function capitalize (str) {
  return str[0].toUpperCase() + str.slice(1).toLowerCase();
}
