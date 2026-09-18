import { invoke } from "@tauri-apps/api/core";
import type { RuntimeStatus } from "./runtime-model";

export async function getRuntimeStatus(): Promise<RuntimeStatus> {
  return invoke<RuntimeStatus>("runtime_status");
}

export async function startRuntime(): Promise<RuntimeStatus> {
  return invoke<RuntimeStatus>("runtime_start");
}

export async function stopRuntime(): Promise<RuntimeStatus> {
  return invoke<RuntimeStatus>("runtime_stop");
}
