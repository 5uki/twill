export type ExtractActionType = "code" | "link";

export function getExtractActionHint(
  type: ExtractActionType,
  copied: boolean,
): string | null {
  if (copied) {
    return null;
  }

  return type === "code" ? "Copy code" : "Open link";
}
