import * as monaco from 'monaco-editor';

type IStandaloneThemeData = monaco.editor.IStandaloneThemeData;

const lightTheme: IStandaloneThemeData = {
  base: 'vs',
  inherit: true,
  rules: [
    // Comments
    { token: 'comment.line.number-sign.toml', foreground: '#6e7487' }, // muted-foreground

    // Table headers
    { token: 'punctuation.definition.table', foreground: '#8b5cf6', fontStyle: 'bold' }, // primary
    { token: 'entity.name.section.table', foreground: '#7662b5', fontStyle: 'bold' }, // primary darker
    { token: 'punctuation.definition.table.array', foreground: '#8b5cf6', fontStyle: 'bold' }, // primary
    { token: 'entity.name.section.table.array', foreground: '#7662b5', fontStyle: 'bold' }, // primary darker

    // Keys and values
    { token: 'key', foreground: '#4b5383' }, // foreground
    { token: 'string.quoted', foreground: '#6366f1' }, // accent-foreground
    { token: 'string.escape', foreground: '#8b5cf6' }, // primary
    { token: 'constant.numeric.integer.toml', foreground: '#8b5cf6' }, // primary
    { token: 'constant.numeric.float.toml', foreground: '#8b5cf6' }, // primary
    { token: 'constant.other.boolean.toml', foreground: '#ec4899' }, // chart-1
    { token: 'constant.other.date.toml', foreground: '#0ea5e9' }, // chart-2
    { token: 'constant.other.datetime.toml', foreground: '#0ea5e9' }, // chart-2
    { token: 'constant.other.datetime-with-timezone.toml', foreground: '#0ea5e9' }, // chart-2
  ],
  colors: {
    'editor.background': '#f0f2f5', // background: hsl(220, 23%, 95%)
    'editor.foreground': '#4c5566', // foreground: hsl(234, 16%, 35%)
    'editor.lineHighlightBackground': '#e5e7eb', // border: hsl(210, 40%, 96.1%)
    'editorLineNumber.foreground': '#9ca3af', // muted-foreground: hsl(215, 8.4%, 59.8%)
    'editorLineNumber.activeForeground': '#374151', // foreground: hsl(222.2, 84%, 4.9%)
    'editorCursor.foreground': '#6366f1', // primary: hsl(240, 84.2%, 60.2%)
    'editor.selectionBackground': '#e0e7ff', // accent: hsl(221, 100%, 94.9%)
    'editor.wordHighlightBackground': '#e0e7ff', // accent: hsl(221, 100%, 94.9%)
    'editor.findMatchBackground': '#d1d5db', // border: hsl(214.3, 31.8%, 91.4%)
    'editor.findMatchHighlightBackground': '#e5e7eb', // border: hsl(210, 40%, 96.1%)
    'editorIndentGuide.background': '#e5e7eb', // border: hsl(210, 40%, 96.1%)
    'editorIndentGuide.activeBackground': '#d1d5db', // ring: hsl(215, 20.2%, 65.1%)
    'editorGutter.background': '#f0f2f5', // background: hsl(220, 23%, 95%)
    'editorGutter.modifiedBackground': '#8b5cf6', // primary: hsl(266, 85%, 58%)
    'editorGutter.addedBackground': '#10b981', // success color
    'editorGutter.deletedBackground': '#ef4444', // destructive
    'scrollbarSlider.background': '#c7d2fe33', // accent with opacity
    'scrollbarSlider.hoverBackground': '#c7d2fe55', // accent with opacity
    'scrollbarSlider.activeBackground': '#c7d2fe77', // accent with opacity

    // Additional colors
    'errorForeground': '#ef4444', // destructive
    'descriptionForeground': '#6b7280', // muted-foreground
  },
};

const darkTheme: IStandaloneThemeData = {
  base: 'vs-dark',
  inherit: true,
  rules: [
    // Comments
    { token: 'comment.line.number-sign.toml', foreground: '#8e9ac0' }, // muted-foreground

    // Table headers
    { token: 'punctuation.definition.table', foreground: '#c4b5fd', fontStyle: 'bold' }, // primary (sidebar-primary)
    { token: 'entity.name.section.table', foreground: '#bca8ff', fontStyle: 'bold' }, // primary lighter
    { token: 'punctuation.definition.table.array', foreground: '#c4b5fd', fontStyle: 'bold' }, // primary
    { token: 'entity.name.section.table.array', foreground: '#bca8ff', fontStyle: 'bold' }, // primary lighter

    // Keys and values
    { token: 'key', foreground: '#d5e3ff' }, // foreground
    { token: 'string.quoted', foreground: '#a5b4fc' }, // accent-foreground
    { token: 'string.escape', foreground: '#c4b5fd' }, // primary
    { token: 'constant.numeric.integer.toml', foreground: '#c4b5fd' }, // primary
    { token: 'constant.numeric.float.toml', foreground: '#c4b5fd' }, // primary
    { token: 'constant.other.boolean.toml', foreground: '#fb7185' }, // chart-1
    { token: 'constant.other.date.toml', foreground: '#38bdf8' }, // chart-2
    { token: 'constant.other.datetime.toml', foreground: '#38bdf8' }, // chart-2
    { token: 'constant.other.datetime-with-timezone.toml', foreground: '#38bdf8' }, // chart-2
  ],
  colors: {
    'editor.background': '#1e1e2e', // background
    'editor.foreground': '#e6e9ef', // foreground
    'editor.lineHighlightBackground': '#313244', // surface-0
    'editorLineNumber.foreground': '#6c7086', // muted-foreground
    'editorLineNumber.activeForeground': '#cad3f5', // text
    'editorCursor.foreground': '#f5e0dc', // rosewater
    'editor.selectionBackground': '#585b70', // surface-2
    'editor.wordHighlightBackground': '#585b70', // surface-2
    'editor.findMatchBackground': '#45475a', // surface-1
    'editor.findMatchHighlightBackground': '#585b70', // surface-2
    'editorIndentGuide.background': '#45475a', // surface-1
    'editorIndentGuide.activeBackground': '#585b70', // surface-2
    'editorGutter.background': '#1e1e2e', // background
    'editorGutter.modifiedBackground': '#cba6f7', // primary
    'editorGutter.addedBackground': '#a6e3a1', // success color
    'editorGutter.deletedBackground': '#f38ba8', // destructive
    'scrollbarSlider.background': '#45475a33', // accent with opacity
    'scrollbarSlider.hoverBackground': '#cba6f7aa', // primary with opacity
    'scrollbarSlider.activeBackground': '#cba6f7cc', // primary with opacity

    // Additional colors
    'errorForeground': '#f38ba8', // destructive
    'descriptionForeground': '#b7bdf8', // muted-foreground
  },
};

export function defineTheme() {
  monaco.editor.defineTheme('sithra-light', lightTheme);
  monaco.editor.defineTheme('sithra-dark', darkTheme);
}

defineTheme()