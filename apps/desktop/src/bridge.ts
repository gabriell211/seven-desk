import { invoke, isTauri } from "@tauri-apps/api/core";

export interface LocalStatus {
  app_name: string;
  app_version: string;
  operating_system: string;
  architecture: string;
  device_id: string | null;
  host_service_available: boolean;
}

export interface BackendStatus {
  status: "ok";
  release: string;
  protocol: number;
  production_ready: boolean;
}

export async function getLocalStatus(): Promise<LocalStatus | null> {
  if (!isTauri()) return null;
  return invoke<LocalStatus>("local_status");
}

export async function getBackendStatus(signal?: AbortSignal): Promise<BackendStatus | null> {
  // Developer control plane only; never expose an unauthenticated remote endpoint.
  const response = await fetch("http://127.0.0.1:43821/healthz", {
    method: "GET",
    signal,
  });
  if (!response.ok) throw new Error("Control plane indisponível");
  const payload: unknown = await response.json();
  if (!payload || typeof payload !== "object") return null;
  if (!("status" in payload) || payload.status !== "ok") return null;
  if (!("production_ready" in payload) || typeof payload.production_ready !== "boolean") return null;
  return payload as BackendStatus;
}
