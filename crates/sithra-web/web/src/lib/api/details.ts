export interface PluginDetails {
  id: string;
  name: string;
  version: string;
  path: string;
  enable: boolean;
  args: string[];
  config: any;
  schema: any;
  toml_str?: string;
  running: boolean;
}

export const path = "/api/plg_details";
