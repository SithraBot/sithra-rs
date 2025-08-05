import * as monaco from 'monaco-editor';
import * as R from 'ramda'
export interface TomlError {
  name: string;
  message: string;
  line: number;
  col: number;
}

export const isParseError = (err: unknown): err is TomlError => {
  return (!!err) && (err as TomlError).name === 'TomlError';
};

export function getMarkersFromError(
  error: TomlError,
): monaco.editor.IMarkerData[] {
  const line = error.line + 1;
  const column = error.col + 1;

  return [
    {
      startColumn: column,
      startLineNumber: line,
      endColumn: column,
      endLineNumber: line,
      message: error.message,
      severity: monaco.MarkerSeverity.Error,
    },
  ];
}

export const debounce = R.curry(<T extends Array<any>, N>(wait: number, fn: (this: N, ...args: T) => void) => {
  let timeout: number | undefined;
  return function (this: N, ...args: T) {
    const context = this;
    const later = () => {
      timeout = undefined;
      fn.apply(context, args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait) as unknown as number;
  }
});

export const tomlCheckDebounce = debounce(200);
