export type MailSecurity = "none" | "start_tls" | "tls";

export interface MailServerConfig {
  host: string;
  port: number;
  security: MailSecurity;
}

export interface AddAccountInput {
  display_name: string;
  email: string;
  login: string;
  imap: MailServerConfig;
  smtp: MailServerConfig;
}

export interface AccountConnectionTestInput {
  display_name: string;
  email: string;
  login: string;
  imap: MailServerConfig;
  smtp: MailServerConfig;
}

export interface AccountSummary {
  id: string;
  display_name: string;
  email: string;
  login: string;
  imap: MailServerConfig;
  smtp: MailServerConfig;
}

export type AccountConnectionStatus = "passed" | "failed";
export type AccountConnectionCheckTarget = "identity" | "imap" | "smtp";

export interface AccountConnectionCheck {
  target: AccountConnectionCheckTarget;
  status: AccountConnectionStatus;
  message: string;
}

export interface AccountConnectionTestResult {
  status: AccountConnectionStatus;
  summary: string;
  checks: AccountConnectionCheck[];
}
