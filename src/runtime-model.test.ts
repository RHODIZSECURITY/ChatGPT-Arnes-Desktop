import { describe, expect, it } from "vitest";
import { runtimeReadiness, type RuntimeStatus } from "./runtime-model";

const ready: RuntimeStatus = {
  platform_supported: true,
  wsl_available: true,
  distro_installed: true,
  docker_ready: true,
  service_state: "active",
  detail: null,
};

describe("runtimeReadiness", () => {
  it("requires every runtime layer to be ready", () => {
    expect(runtimeReadiness(ready)).toBe("ready");
    expect(runtimeReadiness({ ...ready, docker_ready: false })).toBe("partial");
    expect(runtimeReadiness({ ...ready, distro_installed: false })).toBe("partial");
  });

  it("marks an unsupported host or missing WSL unavailable", () => {
    expect(runtimeReadiness({ ...ready, platform_supported: false })).toBe("unavailable");
    expect(runtimeReadiness({ ...ready, wsl_available: false })).toBe("unavailable");
  });
});
