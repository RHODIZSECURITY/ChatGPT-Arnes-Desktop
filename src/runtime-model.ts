export type ServiceState =
  | "active"
  | "inactive"
  | "failed"
  | "unknown"
  | "not_installed";

export interface RuntimeStatus {
  platform_supported: boolean;
  wsl_available: boolean;
  distro_installed: boolean;
  docker_ready: boolean;
  service_state: ServiceState;
  detail: string | null;
}

export function runtimeReadiness(status: RuntimeStatus): "ready" | "partial" | "unavailable" {
  if (
    status.platform_supported &&
    status.wsl_available &&
    status.distro_installed &&
    status.docker_ready &&
    status.service_state === "active"
  ) {
    return "ready";
  }

  if (!status.platform_supported || !status.wsl_available) {
    return "unavailable";
  }

  return "partial";
}

export function stateLabel(value: boolean): string {
  return value ? "Ready" : "Not ready";
}
