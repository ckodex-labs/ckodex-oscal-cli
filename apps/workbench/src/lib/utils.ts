import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function truncateHash(hash: string, len = 8): string {
  if (!hash || hash.length <= len) return hash;
  return `${hash.slice(0, len)}…`;
}
