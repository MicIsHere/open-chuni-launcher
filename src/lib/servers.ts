export type ServerPresetId = "rinnet" | "munet" | "custom";

export interface ServerConfig {
  preset: ServerPresetId;
  customDns: string;
  customAimeDb: string;
  keychip: string;
}

export interface ServerPreset {
  id: Exclude<ServerPresetId, "custom">;
  label: string;
  dnsDefault: string;
  dnsAimeDb?: string;
}

export const SERVER_PRESETS: ServerPreset[] = [
  { id: "rinnet", label: "RinNET", dnsDefault: "aqua.naominet.live" },
  {
    id: "munet",
    label: "MuNET",
    dnsDefault: "play.mumur.net",
    dnsAimeDb: "aime.mumur.net",
  },
];

const KEYCHIP_PATTERN = /^A\d{2}([EX])-(01|20)[ABCDU]\d{8}$/;

export function isValidKeychip(value: string): boolean {
  return KEYCHIP_PATTERN.test(value.trim());
}

export interface ResolvedServerDns {
  dnsDefault: string;
  dnsAimeDb?: string;
}

export function resolveServerDns(config: ServerConfig): ResolvedServerDns {
  const preset = SERVER_PRESETS.find((item) => item.id === config.preset);
  if (config.preset !== "custom" && preset) {
    return { dnsDefault: preset.dnsDefault, dnsAimeDb: preset.dnsAimeDb };
  }
  const resolved: ResolvedServerDns = { dnsDefault: config.customDns.trim() };
  if (config.customAimeDb.trim()) {
    resolved.dnsAimeDb = config.customAimeDb.trim();
  }
  return resolved;
}

export function validateServerConfig(config: ServerConfig): string | undefined {
  if (config.preset === "custom" && !config.customDns.trim()) {
    return "config.server.dnsRequired";
  }
  if (!isValidKeychip(config.keychip)) {
    return "config.keychip.invalid";
  }
  return undefined;
}

export interface ServerPatch {
  dnsDefault: string;
  dnsAimeDb?: string;
  keychip: string;
}

export function buildServerPatch(config: ServerConfig): ServerPatch {
  const dns = resolveServerDns(config);
  const patch: ServerPatch = {
    dnsDefault: dns.dnsDefault,
    keychip: config.keychip.trim(),
  };
  if (dns.dnsAimeDb) patch.dnsAimeDb = dns.dnsAimeDb;
  return patch;
}
