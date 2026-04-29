import { formatDistanceToNow } from "date-fns";
import { REGION_COORDINATES } from "@/lib/region-coordinates";

export type NormalizedNodeStatus = "online" | "degraded" | "offline" | "unknown";

export type NodeHealthMapNode = {
  nodeId: string;
  networkId: string;
  networkName: string;
  status: Exclude<NormalizedNodeStatus, "unknown">;
  region: string;
  coordinates: [number, number];
  knownRegion: boolean;
  lastSeenEpochSecs: number;
  cpuAvailablePct?: number | null;
  ramAvailableMb?: number | null;
};

type NodeLocationLike = {
  node_id?: string;
  id?: string;
  region?: string | null;
  lat?: number | null;
  lng?: number | null;
  latitude?: number | null;
  longitude?: number | null;
};

const STATUS_COLORS: Record<Exclude<NormalizedNodeStatus, "unknown">, string> = {
  online: "#22c55e",
  degraded: "#f59e0b",
  offline: "#ef4444",
};

function hashString(value: string) {
  let hash = 0;

  for (let index = 0; index < value.length; index += 1) {
    hash = (hash << 5) - hash + value.charCodeAt(index);
    hash |= 0;
  }

  return Math.abs(hash);
}

export function normalizeRegionKey(region?: string | null) {
  return region?.trim().toLowerCase().replace(/\s+/g, "-").replace(/_/g, "-") ?? "";
}

export function getRegionCoordinates(region?: string | null) {
  const normalized = normalizeRegionKey(region);

  if (!normalized) {
    return undefined;
  }

  const directMatch = REGION_COORDINATES[normalized];
  if (directMatch) {
    return directMatch;
  }

  const parts = normalized.split("-");
  if (parts.length >= 3) {
    const regionAlias = parts.slice(0, 2).join("-");
    return REGION_COORDINATES[regionAlias];
  }

  return undefined;
}

export function normalizeNodeStatus(status?: string | null): Exclude<NormalizedNodeStatus, "unknown"> {
  const normalized = (status ?? "").trim().toLowerCase();

  if (normalized === "offline") {
    return "offline";
  }

  if (normalized === "draining" || normalized === "preempting" || normalized === "degraded") {
    return "degraded";
  }

  return "online";
}

export function jitterCoords([lng, lat]: [number, number], seed: string) {
  const xSeed = hashString(`${seed}:lng`) % 401;
  const ySeed = hashString(`${seed}:lat`) % 401;
  const lngOffset = xSeed / 100 - 2;
  const latOffset = ySeed / 100 - 2;

  return [lng + lngOffset, lat + latOffset] as [number, number];
}

export function resolveNodeCoordinates(node: NodeLocationLike) {
  const explicitLatitude = node.latitude ?? node.lat;
  const explicitLongitude = node.longitude ?? node.lng;

  if (Number.isFinite(explicitLatitude) && Number.isFinite(explicitLongitude)) {
    return {
      coordinates: [explicitLongitude as number, explicitLatitude as number] as [number, number],
      knownRegion: true,
    };
  }

  const regionCoordinates = getRegionCoordinates(node.region);

  if (regionCoordinates) {
    const seed = node.node_id ?? node.id ?? node.region ?? "node";

    return {
      coordinates: jitterCoords(regionCoordinates, seed),
      knownRegion: true,
    };
  }

  return {
    coordinates: [0, 0] as [number, number],
    knownRegion: false,
  };
}

export function getStatusColor(status: NormalizedNodeStatus, knownRegion = true) {
  if (!knownRegion || status === "unknown") {
    return "#94a3b8";
  }

  return STATUS_COLORS[status];
}

export function formatHeartbeat(epochSecs?: number | null) {
  if (!epochSecs || !Number.isFinite(epochSecs)) {
    return "unknown";
  }

  return formatDistanceToNow(new Date(epochSecs * 1000), { addSuffix: true });
}

export function toNodeHealthMapNode(params: {
  nodeId: string;
  networkId: string;
  networkName: string;
  status: string | null | undefined;
  region?: string | null;
  lastSeenEpochSecs: number;
  cpuAvailablePct?: number | null;
  ramAvailableMb?: number | null;
  lat?: number | null;
  lng?: number | null;
  latitude?: number | null;
  longitude?: number | null;
}) {
  const location = resolveNodeCoordinates({
    id: params.nodeId,
    node_id: params.nodeId,
    region: params.region,
    lat: params.lat,
    lng: params.lng,
    latitude: params.latitude,
    longitude: params.longitude,
  });

  return {
    nodeId: params.nodeId,
    networkId: params.networkId,
    networkName: params.networkName,
    status: normalizeNodeStatus(params.status),
    region: params.region?.trim() || "unknown",
    coordinates: location.coordinates,
    knownRegion: location.knownRegion,
    lastSeenEpochSecs: params.lastSeenEpochSecs,
    cpuAvailablePct: params.cpuAvailablePct,
    ramAvailableMb: params.ramAvailableMb,
  } satisfies NodeHealthMapNode;
}