import { webLightTheme, type Theme } from "@fluentui/react-components";

// We will use the refined default webLightTheme from Fluent UI, which is a professional Blue.
// We just add some custom font stacks to keep it clean and readable.
export const twillTheme: Theme = {
  ...webLightTheme,
  fontFamilyBase: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
  fontFamilyNumeric: "'JetBrains Mono', 'Cascadia Code', Consolas, monospace",
  
  // Slightly soften the backgrounds for a cleaner look
  colorNeutralBackground1: "#ffffff",
  colorNeutralBackground2: "#f3f4f6", // very light gray for sidebar/background
  colorNeutralBackground3: "#e5e7eb",
};
