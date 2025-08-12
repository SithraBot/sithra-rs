export const path = "/api/list_files";

export interface FileEntry {
  path: string;
  name: string;
  ty: FileType;
}

export type FileType = "File" | "Dir";

export interface ListFiles {
  path: string;
}
