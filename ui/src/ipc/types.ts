import type { AppInfo } from '../bindings/AppInfo';

/** Every call from the UI into the Rust core. One method per Tauri command. */
export interface Commands {
  /** Name, version and platform of the running app (command `app_info`). */
  appInfo: () => Promise<AppInfo>;
}
