const mobileUserAgentPattern =
  /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i;

export type TitlebarWindowAction = "maximize" | "restore";

export interface TitlebarPlatformState {
  isMac: boolean;
  isMobile: boolean;
}

export function getTitlebarPlatformState(
  userAgent: string,
): TitlebarPlatformState {
  const isMobile = mobileUserAgentPattern.test(userAgent);

  return {
    isMac: userAgent.includes("Mac OS X") && !isMobile,
    isMobile,
  };
}

export function getTitlebarWindowAction(
  isMaximized: boolean,
): TitlebarWindowAction {
  return isMaximized ? "restore" : "maximize";
}
