export interface PluginInfo {
  id: string;
  running: boolean;
}

export interface Data {
  plgsInfo: PluginInfo[];
  isRegistered: boolean;
}

export const path = "/api/plgs_info";
