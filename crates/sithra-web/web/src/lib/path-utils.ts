export function shortenPath(filePath: string, maxLength: number): string {
  if (filePath.length <= maxLength) {
    return filePath;
  }

  const parts = filePath.split('/');

  const isAbsolute = parts[0] === '';

  const fileName = parts[parts.length - 1];

  let dirParts = parts.slice(0, -1);

  if (isAbsolute) {
    dirParts = dirParts.slice(1);
  }

  for (let i = 0; i < dirParts.length; i++) {
    const shortenedDirs = dirParts.map((part, index) => {
      if (index <= i && part.length > 0) {
        return part[0];
      }
      return part;
    });

    let result = '';
    if (isAbsolute) {
      result = '/';
    }

    if (shortenedDirs.length > 0) {
      result += shortenedDirs.join('/');
      if (fileName) {
        result += '/' + fileName;
      }
    } else {
      result += fileName;
    }

    if (result.length <= maxLength) {
      return result;
    }
  }

  let finalResult = '';
  if (isAbsolute) {
    finalResult = '/';
  }

  const allShortenedDirs = dirParts.map(part => part.length > 0 ? part[0] : '').filter(part => part);
  if (allShortenedDirs.length > 0) {
    finalResult += allShortenedDirs.join('/') + '/' + fileName;
  } else {
    finalResult += fileName;
  }

  return finalResult;
}